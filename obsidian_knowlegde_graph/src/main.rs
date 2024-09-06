mod data;

use data::{data, lockbookdata, Graph};
use eframe::{egui, App, Frame};
use egui::emath::Numeric;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{f32, time::Instant};

struct Grid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<usize>>, // Stores node indices in grid cells
}

#[derive(Default)]
struct KnowledgeGraphApp {
    graph: Graph,
    positions: Vec<egui::Pos2>,
    forces: Vec<egui::Vec2>,
    zoom_factor: f32,
    last_screen_size: egui::Vec2,
    cursor_loc: egui::Vec2,
    debug: String,
    is_dragging: bool,
    last_drag_pos: Option<egui::Pos2>,
    layout_time: f64,
    graph_complete: bool,
    layout_started: bool,
    iteration: usize, // Track the current iteration in the layout
    change: egui::Vec2,
    last_change: egui::Vec2,
    running: bool,
    wait: f64,
}

impl Grid {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            grid: HashMap::new(),
        }
    }

    fn insert_node(&mut self, pos: egui::Pos2, index: usize) {
        let grid_pos = self.get_grid_pos(pos);
        self.grid
            .entry(grid_pos)
            .or_insert_with(Vec::new)
            .push(index);
    }

    fn get_grid_pos(&self, pos: egui::Pos2) -> (i32, i32) {
        let x = (pos.x / self.cell_size).floor() as i32;
        let y = (pos.y / self.cell_size).floor() as i32;
        (x, y)
    }

    fn get_neighboring_cells(&self, pos: egui::Pos2) -> Vec<&Vec<usize>> {
        let grid_pos = self.get_grid_pos(pos);
        let mut neighboring_cells = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell) = self.grid.get(&(grid_pos.0 + dx, grid_pos.1 + dy)) {
                    neighboring_cells.push(cell);
                }
            }
        }
        neighboring_cells
    }

    fn clear(&mut self) {
        self.grid.clear();
    }
}

impl KnowledgeGraphApp {
    fn new(graph: Graph) -> Self {
        let positions = vec![egui::Pos2::ZERO; graph.len()];
        let forces = vec![egui::Vec2::ZERO; graph.len()];
        Self {
            graph,
            positions,
            forces,
            zoom_factor: 1.0,
            last_screen_size: egui::Vec2::new(800.0, 600.0),
            cursor_loc: egui::Vec2::ZERO,
            debug: String::from("no single touch"),
            is_dragging: false,
            last_drag_pos: None,
            layout_time: 0.0,
            graph_complete: false,
            layout_started: false,
            iteration: 0,
            change: egui::Vec2::ZERO,
            last_change: egui::Vec2::ZERO,
            running: true,
            wait: 100.0,
        }
    }

    fn initialize_positions(&mut self) {
        let width = 800.0;
        let height = 600.0;
        let num_nodes = self.graph.len() as f32;
        let radius = ((f32::min(width, height) / 2.0) - 50.0) * (num_nodes.sqrt() / 10.0); // Adjust based on number of nodes
        let angle_step = 2.0 * std::f32::consts::PI / num_nodes;

        self.positions = (0..num_nodes as usize)
            .map(|i| {
                let angle = i as f32 * angle_step;
                egui::Pos2::new(
                    width / 2.0 + radius * angle.cos(),
                    height / 2.0 + radius * angle.sin(),
                )
            })
            .collect();
    }

    fn apply_spring_layout(&mut self) {
        let width = 700.0;
        let height = 500.0;
        let num_nodes = self.graph.len() as f32;

        // Define spring layout constants
        let k_spring = 0.01; // Spring constant for attraction
        let k_repel = 500.0; // Repulsion constant
        let c = 0.01; // Damping factor to control movement

        // Limit the maximum movement per iteration
        let max_movement = 50.0;

        // Set up grid-based approximation
        let cell_size = (width * height / num_nodes).sqrt();
        let mut grid = Grid::new(cell_size);

        // Insert nodes into the grid based on their positions
        for (i, &pos) in self.positions.iter().enumerate() {
            grid.insert_node(pos, i);
        }

        // Reset forces to zero before recalculating
        for i in 0..self.graph.len() {
            self.forces[i] = egui::Vec2::ZERO;
        }

        // Calculate repulsive forces between all nodes
        for i in 0..self.graph.len() {
            let pos_i = self.positions[i];

            // Get neighboring cells for the current node
            for cell in grid.get_neighboring_cells(pos_i) {
                for &j in cell {
                    if i != j {
                        let delta = self.positions[i] - self.positions[j];
                        let distance = delta.length().max(0.01); // Avoid division by zero

                        // Calculate repulsive force based on the distance between nodes
                        let repulsive_force = (k_repel) / (distance * distance); // Repulsive force inversely proportional to distance
                        let repulsion = delta.normalized() * repulsive_force;

                        // Apply equal and opposite forces to both nodes
                        self.forces[i] += repulsion;
                        self.forces[j] -= repulsion;
                    }
                }
            }
        }

        // Apply attractive forces between connected nodes
        for node in &self.graph {
            for &link in &node.links {
                let delta = self.positions[node.id] - self.positions[link];
                let distance = delta.length().max(0.01);

                // Calculate attractive force between connected nodes
                let attractive_force = k_spring * (distance * distance); // Spring-based attractive force
                let attraction = delta.normalized() * attractive_force;

                // Apply the attraction between connected nodes
                self.forces[node.id] -= attraction;
                self.forces[link] += attraction;
            }
        }

        // Apply forces to update positions, but limit movement to avoid instability
        for i in 0..self.graph.len() {
            let force_magnitude = self.forces[i].length();

            // Limit the movement per iteration to avoid nodes moving too fast
            let movement = if force_magnitude > max_movement {
                self.forces[i] * (max_movement / force_magnitude)
            } else {
                self.forces[i]
            };

            self.positions[i] += movement * c;

            // Removed the clamping to allow free movement beyond boundaries
            // self.positions[i].x = self.positions[i].x.clamp(0.0, width);
            // self.positions[i].y = self.positions[i].y.clamp(0.0, height);
        }

        // Increment the iteration on every pass
        self.iteration += 1;

        // Check for convergence by monitoring the total change in position
        let total_change = self.forces.iter().map(|f| f.length()).sum::<f32>();

        if total_change < 20.0 || self.iteration >= 30000 {
            // Updated to 1500.0 as per your request
            // Mark graph as complete if no movement or reached iteration limit
            self.graph_complete = true;
            self.running = false;
        }

        println!(
            "Iteration: {}, Total Change: {}",
            self.iteration, total_change
        );

        // Clear the grid for the next iteration
        grid.clear();

        // Request another update if the layout is not complete
        if !self.graph_complete {
            // ctx.request_repaint();  // Uncomment if using a UI context like egui
        }
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui, screen_size: egui::Vec2) {
        let center = screen_size / 2.0;
        let radius = (15.0 * self.zoom_factor) / ((self.graph.len() as f32).sqrt() / 3.0).max(1.0);

        self.graph.iter().enumerate().for_each(|(i, node)| {
            let rgb_color = egui::Color32::from_rgb(
                (node.color[0] * 355.0) as u8,
                (node.color[1] * 355.0) as u8,
                (node.color[2] * 355.0) as u8,
            );
            let pos = self.positions[i].to_vec2();
            ui.painter().circle(
                pos.to_pos2(),
                radius,
                rgb_color,
                egui::Stroke::new(2.0 * self.zoom_factor, egui::Color32::BLACK),
            );

            node.links
                .iter()
                .filter_map(|&link| {
                    self.positions.get(link).map(|&target_pos| {
                        let target_pos = target_pos.to_vec2();
                        ui.painter().line_segment(
                            [pos.to_pos2(), target_pos.to_pos2()],
                            egui::Stroke::new(1.0 * self.zoom_factor, egui::Color32::GRAY),
                        );
                    })
                })
                .for_each(drop);
            if radius > 15.0 {
                let font_id = egui::FontId::proportional(radius);
                ui.painter().text(
                    pos.to_pos2(),
                    egui::Align2::CENTER_CENTER,
                    &node.title,
                    font_id,
                    egui::Color32::WHITE,
                );
            }
        });

        self.last_screen_size = screen_size;
    }

    fn zoomed(&mut self, zoom: f32) {
        self.positions = self
            .positions
            .iter()
            .map(|&pos| (self.cursor_loc + ((pos.to_vec2() - self.cursor_loc) * zoom)).to_pos2())
            .collect();
    }

    fn draged(&mut self, offset: egui::Vec2) {
        self.positions = self
            .positions
            .iter()
            .map(|&pos| (pos.to_vec2() + offset).to_pos2())
            .collect();
    }

    fn label_subgraphs(&mut self) {
        let mut bluecol = 1.0;
        let mut redcol = 0.1;
        let mut greencol = 0.5;

        for i in 0..self.graph.len() {
            if self.graph[i].color[2] == 0.0 {
                self.dfs(i, bluecol, redcol, greencol);

                // Update color values in a way that keeps them within a visible range
                bluecol = (bluecol * 0.7 + 0.2) % 1.0;
                redcol = (redcol * 1.5 + 0.3) % 1.0;
                greencol = (greencol * 1.3 + 0.4) % 1.0;
            }
        }
    }

    fn dfs(&mut self, node_id: usize, col: f32, redcol: f32, greencol: f32) {
        let links_to_visit: Vec<usize> = {
            let node = &self.graph[node_id];
            node.links
                .iter()
                .filter_map(|&id| {
                    if self.graph[id].color[2] == 0.0 {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Update the color of the current node
        self.graph[node_id].color[0] = redcol;
        self.graph[node_id].color[1] = greencol;
        self.graph[node_id].color[2] = col;

        for id in links_to_visit {
            self.graph[id].color[0] = redcol;
            self.graph[id].color[1] = greencol;
            self.graph[id].color[2] = col;
            self.dfs(id, col, redcol, greencol);
        }
    }
}

impl eframe::App for KnowledgeGraphApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Knowledge Graph");
            ui.text_edit_singleline(&mut self.debug);

            // Display the layout computation time
            ui.label(format!(
                "Layout computation time: {:.2} seconds",
                self.layout_time
            ));

            let screen_size = ui.available_size();

            if !self.layout_started {
                // Initialize positions and draw the initial graph
                self.initialize_positions();
                self.layout_started = true;
            } else if !self.graph_complete {
                // Apply the spring layout
                self.apply_spring_layout();
            }

            // Always draw the graph, even as it updates
            self.draw_graph(ui, screen_size);

            // Request another repaint to continue the layout process
            if !self.graph_complete {
                ctx.request_repaint();
            }
            if self.running {
                self.debug = String::from("running");
            } else {
                let start_time = Instant::now();
                self.debug = String::from("done");
            }
        });
        if !self.running {
            // Handle dragging and zooming (same as before)
            let pointer = ctx.input(|i| i.pointer.clone());
            if pointer.any_down() {
                if let Some(current_pos) = pointer.interact_pos() {
                    if !self.is_dragging {
                        self.is_dragging = true;
                        self.last_drag_pos = Some(current_pos);
                        // self.debug = String::from("clicked");
                    } else if let Some(last_pos) = self.last_drag_pos {
                        // self.debug = String::from("dragged");
                        let delta = current_pos - last_pos;
                        self.draged(delta);
                        self.last_drag_pos = Some(current_pos);
                    }
                }
            } else {
                // self.debug = String::from("clicked");
                self.is_dragging = false;
                self.last_drag_pos = None;
            }

            if let Some(touches) = ctx.input(|i| i.multi_touch()) {
                self.debug = touches.num_touches.to_string();
            }

            let mut zoom = 1.0;
            if ctx.input(|i| i.key_pressed(egui::Key::Equals)) {
                self.zoom_factor *= 1.1;
                zoom = 1.1;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Minus)) {
                self.zoom_factor *= 0.9;
                zoom = 0.9;
            }
            if zoom != 1.0 {
                if let Some(cursor) = ctx.input(|i| i.pointer.hover_pos()) {
                    self.cursor_loc = cursor.to_vec2();
                    self.zoomed(zoom);
                }
            }
        }
    }
}

fn main() {
    let graph = lockbookdata();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    // Start the counting thread
    thread::spawn(move || {
        let mut seconds = 0;
        while !stop_flag_clone.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            seconds += 1;
            println!("{} second", seconds);
        }
    });

    let mut app = KnowledgeGraphApp::new(graph);
    app.label_subgraphs();
    stop_flag.store(true, Ordering::SeqCst);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Knowledge Graph App",
        native_options,
        Box::new(|_cc| Box::new(app)),
    )
    .unwrap();
}
