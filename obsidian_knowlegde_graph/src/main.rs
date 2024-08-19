mod data;

use data::{data, Graph};
use eframe::{egui, App, Frame};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{f32, time::Instant};

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
            running: false,
        }
    }

    fn initialize_positions(&mut self) {
        let width = 800.0;
        let height = 600.0;
        let num_nodes = self.graph.len() as f32;
        let radius = (f32::min(width, height) / 2.0) - 50.0;
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
        thread::sleep(Duration::from_millis(5));
        self.graph_complete = false;
        self.running = true;
        self.iteration = self.iteration + 1;
        // let start_time = Instant::now(); // Start timing
        // let mut change = egui::Vec2::ZERO;
        // let mut last_change = egui::Vec2::ZERO;
        let width = 800.0;
        let height = 600.0;
        let num_nodes = self.graph.len() as f32;

        // let iterations = 1000 * self.graph.len();
        let k = (width * height / (self.graph.len() as f32)).sqrt() * 0.2;
        let c = 0.005;
        // let mut number = 0;

        // while number != iterations && !converged {
        //     number += 1;
        for i in 0..self.graph.len() {
            self.forces[i] = egui::Vec2::ZERO;
        }

        for row in 0..self.graph.len() {
            for col in row + 1..self.graph.len() {
                let delta = self.positions[row] - self.positions[col];
                let distance = delta.length().max(0.01);
                let repulsive_force = (k * k) / distance * (10.0 / (num_nodes * num_nodes));
                self.forces[row] += delta.normalized() * repulsive_force * 2.0;
            }
        }

        for node in &self.graph {
            for &link in &node.links {
                let delta = self.positions[node.id] - self.positions[link];
                let distance = delta.length().max(0.01);
                let attractive_force = (distance * distance) / k * 0.5;
                self.forces[node.id] -= delta.normalized() * attractive_force;
                self.forces[link] += delta.normalized() * attractive_force;
            }
        }

        for i in 0..self.graph.len() {
            for j in (i + 1)..self.graph.len() {
                let delta = self.positions[i] - self.positions[j];
                let distance = delta.length().max(0.01);
                let min_distance = 100.0;

                if distance < min_distance {
                    let overlap_force = (min_distance - distance) * 0.5;
                    self.forces[i] += delta.normalized() * overlap_force * 2.0;
                    self.forces[j] -= delta.normalized() * overlap_force * 2.0;
                }
            }
        }

        for i in 0..self.graph.len() {
            for node in &self.graph {
                for &link in &node.links {
                    let a = self.positions[node.id];
                    let b = self.positions[link];
                    if node.id != i && link != i {
                        let c = self.positions[i];

                        let ab = b - a;
                        let ac = c - a;
                        let t = ac.dot(ab) / ab.dot(ab);

                        let t = t.clamp(0.0, 1.0);
                        let closest_point = a + ab * t;
                        let delta = c - closest_point;
                        let distance = delta.length().max(0.01);
                        let min_distance = 30.0;

                        if distance < min_distance {
                            let link_force = (min_distance - distance) * 0.5;
                            self.forces[i] += delta.normalized() * link_force;
                        }
                    }
                }
            }

            self.positions[i] += self.forces[i] * c;
            self.change += self.forces[i];
        }

        let totalchange1 = (self.last_change[0].abs() - self.change[0].abs()).abs();
        let totalchange2 = (self.last_change[1].abs() - self.change[1].abs()).abs();
        let sumtch = totalchange1 + totalchange2;

        if (sumtch > -0.0024 && sumtch < 0.0024) && sumtch != 0.0 {
            println!("{:?}", sumtch);
            self.graph_complete = true;
            self.running = false;
            println!("convered");
            println!("{:?}", self.change);
        }
        self.last_change = self.change;
        self.change = egui::Vec2::ZERO;

        // Redraw the UI to reflect the changes
        //ctx.request_repaint();
        //thread::sleep(Duration::from_millis(100));
        //}
        println!("running");
        println!("{:?}", self.iteration);
        // let end_time = Instant::now(); // End timing
        // self.layout_time = (end_time - start_time).as_secs_f64(); // Store the elapsed time
        // self.graph_complete = true;
    }
    fn draw_graph(&mut self, ui: &mut egui::Ui, screen_size: egui::Vec2) {
        let center = screen_size / 2.0;
        let radius = (30.0 * self.zoom_factor) / ((self.graph.len() as f32).sqrt() / 3.0).max(1.0);

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
        for i in 0..self.graph.len() {
            if self.graph[i].color[2] == 0.0 {
                self.dfs(i, bluecol, redcol);
                bluecol *= 0.5;
                redcol += 0.2;
            }
        }
    }

    fn dfs(&mut self, node_id: usize, col: f32, redcol: f32) {
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

        for id in links_to_visit {
            self.graph[id].color[0] = redcol;
            self.graph[id].color[2] = col;
            self.dfs(id, col, redcol);
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
                self.debug = String::from("done");
            }
        });

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

fn main() {
    let graph = data();
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
