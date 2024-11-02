mod data;
use crate::data::LinkNode;
use data::{lockbookdata, Graph};
use eframe::egui;
use egui::epaint::Shape;
use egui::{Color32, Painter, Pos2, Stroke, Vec2};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time;
use std::time::Duration;
use std::{f32, time::Instant};
use std::{thread, usize};

// The makes it the code runs faster making it into grids
struct Grid {
    cell_size: f32,
    grid: HashMap<(i32, i32), Vec<usize>>,
}

// #[derive(Default)]
// The main reason for these are to make global variables that can be accessed through the whole code
struct KnowledgeGraphApp {
    graph: Graph,
    positions: Vec<egui::Pos2>,
    forces: Vec<egui::Vec2>,
    zoom_factor: f32,
    pan: Vec2,
    last_screen_size: egui::Vec2,
    cursor_loc: egui::Vec2,
    debug: String,
    is_dragging: bool,
    last_drag_pos: Option<egui::Pos2>,
    layout_time: f64,
    graph_complete: bool,
    layout_started: bool,
    iteration: usize,
    running: bool,
    directional_links: HashMap<usize, Vec<usize>>,
    animation: bool,
    timer: time::Instant,
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
            pan: Vec2::ZERO,
            last_screen_size: egui::Vec2::new(800.0, 600.0),
            cursor_loc: egui::Vec2::ZERO,
            debug: String::from("no single touch"),
            is_dragging: false,
            last_drag_pos: None,
            layout_time: 0.0,
            graph_complete: false,
            layout_started: false,
            iteration: 0,
            running: true,
            directional_links: HashMap::new(),
            animation: true,
            timer: Instant::now(),
        }
    }

    fn build_directional_links(&mut self) {
        let mut directional_links = HashMap::new();

        for node in &self.graph {
            let mut directional = Vec::new();
            for &link in &node.links {
                if link < self.graph.len() {
                    if !self.graph[link].links.contains(&node.id) {
                        directional.push(link);
                    }
                }
            }
            directional_links.insert(node.id, directional);
        }

        self.directional_links = directional_links;
    }

    fn initialize_positions(&mut self) {
        let width = 800.0;
        let height = 600.0;

        let main_center = Pos2::new(width / 2.0, height / 2.0);

        let cluster_small_radius = 10.0;
        let main_circle_radius = 50.0;

        let mut positions_map = HashMap::new();
        let mut clusters: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
        let mut unlinked_nodes: Vec<usize> = Vec::new();

        for node in &self.graph {
            if node.links.is_empty() {
                unlinked_nodes.push(node.id);
            } else {
                clusters
                    .entry(node.cluster_id)
                    .or_insert_with(Vec::new)
                    .push(node.id);
            }
        }

        let mut largest_cluster_id: Option<usize> = None;
        let mut largest_cluster_size: usize = 0;

        for (cluster_id, node_ids) in &clusters {
            if node_ids.len() > largest_cluster_size {
                largest_cluster_size = node_ids.len();
                largest_cluster_id = *cluster_id;
            }
        }

        let num_multi_clusters = clusters.len();

        if num_multi_clusters > 0 {
            let angle_step_clusters = 2.0 * std::f32::consts::PI / num_multi_clusters as f32;

            for (cluster_id, node_ids) in clusters {
                let number_nodes = node_ids.len();
                let angle_step_nodes = 2.0 * std::f32::consts::PI / number_nodes as f32;
                let mut count: f32 = 0.0;

                let is_largest = Some(cluster_id.unwrap()) == largest_cluster_id;

                let cluster_center = if is_largest {
                    main_center
                } else {
                    let angle = cluster_id.unwrap() as f32 * angle_step_clusters;
                    Pos2::new(
                        main_center.x + main_circle_radius * angle.cos(),
                        main_center.y + main_circle_radius * angle.sin(),
                    )
                };

                for node_id in node_ids {
                    let node_angle = count * angle_step_nodes;
                    let node_pos = Pos2::new(
                        cluster_center.x + cluster_small_radius * node_angle.cos(),
                        cluster_center.y + cluster_small_radius * node_angle.sin(),
                    );
                    positions_map.insert(node_id, node_pos);
                    count += 1.0;
                }
            }
        }

        let total_outer_nodes = unlinked_nodes.len();

        if total_outer_nodes > 0 {
            for (_i, &node_id) in unlinked_nodes.iter().enumerate() {
                let nocluster: Option<usize> = None;
                self.graph[node_id].cluster_id = nocluster;
            }
        }

        self.positions = (0..self.graph.len())
            .map(|i| *positions_map.get(&i).unwrap_or(&main_center))
            .collect();
    }

    fn apply_spring_layout(&mut self) {
        let width = 700.0;
        let height = 500.0;
        let num_nodes = self.graph.len() as f32;
        let len = (self.iteration as f32).sqrt() + 1.0;
        let mut len: usize = len as usize;
        // let len = (len.sqrt() as usize);
        if self.animation {
            len = len + 1;
        } else {
            len = 250000;
        }
        for _n in 0..len {
            let k_spring = 0.0005;
            let k_repel = 3.0;
            // let k_link_nudge = 0.01;
            let c = 0.5;

            let max_movement = 100.0;

            let cell_size = (width * height / num_nodes).sqrt();
            let mut grid = Grid::new(cell_size);

            for (i, &pos) in self.positions.iter().enumerate() {
                grid.insert_node(pos, i);
            }

            for i in 0..self.graph.len() {
                self.forces[i] = egui::Vec2::ZERO;
            }

            //
            for i in 0..self.graph.len() {
                let pos_i = self.positions[i];

                for cell in grid.get_neighboring_cells(pos_i) {
                    for &j in cell {
                        if i != j {
                            let delta = self.positions[i] - self.positions[j];
                            let distance = delta.length().max(0.01);

                            let repulsive_force = k_repel / (distance * distance / 20.0);
                            let repulsion = delta.normalized() * repulsive_force;

                            self.forces[i] += repulsion;
                            self.forces[j] -= repulsion;
                        }
                    }
                }
            }

            for node in &self.graph {
                for &link in &node.links {
                    let delta = self.positions[node.id] - self.positions[link];
                    let distance = delta.length().max(0.01);

                    let attractive_force = k_spring * distance * (distance / 20.0) as f32;
                    let attraction = delta.normalized() * attractive_force;

                    self.forces[node.id] -= attraction;
                    self.forces[link] += attraction;
                }
            }

            // for node in &self.graph {
            //     let pos_node = self.positions[node.id];

            //     for other_node in &self.graph {
            //         for &link in &other_node.links {
            //             if node.id == other_node.id || node.id == link {
            //                 continue;
            //             }

            //             let pos_a = self.positions[other_node.id];
            //             let pos_b = self.positions[link];

            //             let line_vec = pos_b - pos_a;
            //             let node_vec = pos_node - pos_a;

            //             let line_length_sq = line_vec.dot(line_vec).max(0.01);
            //             let t = (node_vec.dot(line_vec) / line_length_sq).clamp(0.0, 1.0);
            //             let closest_point = pos_a + line_vec * t;

            //             let delta = pos_node - closest_point;
            //             let distance = delta.length().max(0.01);

            //             if distance < 30.0 {
            //                 let mut rng = rand::thread_rng();
            //                 let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            //                 let random_direction = egui::Vec2::angled(angle);

            //                 let nudge_magnitude = k_link_nudge * (30.0 - distance);
            //                 let nudge = random_direction * nudge_magnitude;

            //                 self.forces[node.id] += nudge;
            //             }
            //         }
            //     }
            // }

            for i in 0..self.graph.len() {
                let force_magnitude = self.forces[i].length();

                let movement = if force_magnitude > max_movement {
                    self.forces[i] * (max_movement / force_magnitude)
                } else {
                    self.forces[i]
                };

                self.positions[i] += movement * c;
            }

            self.iteration += 1;

            let total_change = self.forces.iter().map(|f| f.length()).sum::<f32>();

            println!("{:?}", total_change);
            if total_change < 0.02 * (num_nodes) || self.iteration >= 250000 {
                self.graph_complete = true;
                self.running = false;
                break;
            }
            println!("{:?}", self.iteration);
            grid.clear();
        }
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui, screen_size: egui::Vec2) {
        let center = Pos2::new(screen_size.x / 2.0, screen_size.y / 2.0);
        let radius = (15.0) / ((self.graph.len() as f32).sqrt() / 3.0).max(1.0);

        let base_size = radius;
        let k = 1.0;

        let node_sizes: Vec<f32> = self
            .graph
            .iter()
            .map(|node| {
                let n = node.links.len() as f32;
                base_size + k * (n + 2.0).sqrt() * self.zoom_factor
            })
            .collect();

        let transformed_positions: Vec<Pos2> = self
            .positions
            .iter()
            .map(|pos| {
                let zoomed = (pos.to_vec2() - center.to_vec2()) * self.zoom_factor;
                let panned = zoomed + self.pan;
                (center.to_vec2() + panned).to_pos2()
            })
            .collect();

        for (i, node) in self.graph.iter().enumerate() {
            for &link in &node.links {
                if let Some(&target_pos) = transformed_positions.get(link) {
                    let size = node_sizes[i];
                    let pos = transformed_positions[i];
                    let target = target_pos;

                    ui.painter().line_segment(
                        [pos, target],
                        Stroke::new(1.0 * self.zoom_factor, Color32::GRAY),
                    );

                    if self.has_directed_link(node.id, self.graph[link].id)
                        && node_sizes[i] > 15.0
                        && cursorin(self.cursor_loc, pos, size)
                    {
                        draw_arrow(
                            ui.painter(),
                            pos,
                            target,
                            Color32::from_rgba_unmultiplied(66, 135, 245, 150), // Semi-transparent blue
                            self.zoom_factor,
                            node_sizes[1],
                        );
                    }
                }
            }
        }

        for (i, node) in self.graph.iter().enumerate() {
            let rgb_color = Color32::from_rgb(
                (node.color[0] * 255.0) as u8,
                (node.color[1] * 255.0) as u8,
                (node.color[2] * 255.0) as u8,
            );

            let size = node_sizes[i];
            let mut text_color = Color32::BLACK;
            let mut text = node.title.clone();
            if node.title.ends_with(".md") {
                text_color = Color32::WHITE;
                text = node.title.trim_end_matches(".md").to_string();
            }
            if node.cluster_id.is_some() {
                let pos = transformed_positions[i];

                ui.painter().circle(
                    pos,
                    size,
                    rgb_color,
                    Stroke::new(0.75 * self.zoom_factor, text_color),
                );

                if size > 15.0 && cursorin(self.cursor_loc, pos, size) {
                    let font_id = egui::FontId::proportional(6.0 * self.zoom_factor); // Adjust font size based on zoom
                    ui.painter().text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        &text,
                        font_id,
                        Color32::WHITE,
                    );
                }
            }
        }

        self.last_screen_size = screen_size;
    }
    fn has_directed_link(&self, from_node: usize, to_node: usize) -> bool {
        if let Some(links) = self.directional_links.get(&from_node) {
            links.contains(&to_node)
        } else {
            false
        }
    }

    fn zoomed(&mut self, zoom: f32) {
        self.positions = self
            .positions
            .iter()
            .map(|&pos| (self.cursor_loc + ((pos.to_vec2() - self.cursor_loc) * zoom)).to_pos2())
            .collect();
    }

    fn label_subgraphs(&mut self) {
        let mut bluecol = 1.0;
        let mut redcol = 0.1;
        let mut greencol = 0.5;

        for i in 0..self.graph.len() {
            if self.graph[i].color[2] == 0.0 {
                if self.graph[i].links.is_empty() {
                    self.graph[i].color = [1.0, 1.0, 1.0];
                } else {
                    self.dfs(i, bluecol, redcol, greencol);

                    bluecol = (bluecol * 0.7 + 0.2) % 1.0;
                    redcol = (redcol * 1.5 + 0.3) % 1.0;
                    greencol = (greencol * 1.3 + 0.4) % 1.0;
                }
            }
        }
    }

    fn label_clusters(&mut self) {
        let mut node_ids: Vec<usize> = Vec::new();

        for node in &self.graph {
            node_ids.push(node.id);
        }
        let mut count = 1;
        println!("{}", node_ids.len());
        for node_id in node_ids {
            println!("Node {}", node_id);
            self.clusters(node_id, count);
            count += 1;
        }
        println!("{:?}", self.graph);
    }

    fn clusters(&mut self, node_id: usize, cluster_id: usize) {
        if self.graph[node_id].cluster_id.is_some() {
            return;
        }

        self.graph[node_id].cluster_id = Some(cluster_id);
        println!("Node {} assigned to cluster {}", node_id, cluster_id);

        let links = self.graph[node_id].links.clone();

        for link in links {
            if link != node_id {
                self.clusters(link, cluster_id);
            }
        }
    }

    fn bidiretional(&mut self) {
        let clonedgraph: &Graph = &self.graph.clone();
        for nodes in clonedgraph {
            let node: usize = nodes.id;
            println!("{:?}", nodes);
            for link in &nodes.links {
                if !clonedgraph[*link].links.contains(&node) {
                    println!("pushed");
                    self.graph[*link].links.push(node)
                }
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            // let is_zoom_modifier = if cfg!(target_os = "macos") {
            //     i.modifiers.mac_cmd
            // } else {
            //     i.modifiers.ctrl
            // };

            // if is_zoom_modifier {
            self.zoom_factor *= i.zoom_delta();
            self.debug = (self.zoom_factor).to_string();
            // let scrolly = i.raw_scroll_delta.y;
            let scroll = i.raw_scroll_delta.to_pos2();
            self.pan += (scroll).to_vec2();
            println!("{:?}", scroll);
            // if scroll != 0.0 {
            //     self.zoom_factor *= 1.0 + scroll * 0.1;

            //     self.zoom_factor = self.zoom_factor.clamp(0.5, 3.0);
            // }
            self.debug = (self.zoom_factor).to_string();
            // }
        });

        if ctx.input(|i| i.pointer.primary_down()) {
            if let Some(current_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                if !self.is_dragging {
                    self.is_dragging = true;
                    self.last_drag_pos = Some(current_pos);
                } else if let Some(last_pos) = self.last_drag_pos {
                    let delta = current_pos - last_pos;

                    self.pan += delta / self.zoom_factor;
                    self.last_drag_pos = Some(current_pos);
                }
            }
        } else {
            self.is_dragging = false;
            self.last_drag_pos = None;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                // Reserve space for the button and text
                ui.allocate_ui_with_layout(
                    Vec2::new(200.0, 50.0),
                    egui::Layout::left_to_right(egui::Align::Min),
                    |ui| {
                        // Display the text to the left of the button
                        ui.label("Animation");

                        // Button logic
                        let _button = egui::Button::new("")
                            .frame(false)
                            .sense(egui::Sense::click());

                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(30.0, 30.0), // Circle size
                            egui::Sense::click(),
                        );

                        // Check if button is clicked
                        if response.clicked() {
                            self.animation = !self.animation;
                        }

                        // Set button color based on pressed state
                        let color = if self.animation {
                            Color32::BLUE
                        } else {
                            Color32::GRAY
                        };

                        // Draw the circle button
                        ui.painter().circle_filled(rect.center(), 5.0, color); // 15.0 is the radius
                    },
                );
            });
            ui.heading("Knowledge Graph");
            ui.text_edit_singleline(&mut self.debug);

            ui.label(format!(
                "Layout computation time: {:.2} seconds",
                self.layout_time
            ));

            let screen_size = ui.available_size();

            // let mut time: Instant = Instant::now();
            if !self.layout_started {
                self.timer = Instant::now();
                self.initialize_positions();
                self.layout_started = true;
            } else if !self.graph_complete {
                self.apply_spring_layout();
                println!("iteration {}", self.iteration);
            }
            // println!("{:?}", time.elapsed());

            self.draw_graph(ui, screen_size);

            if !self.graph_complete {
                ctx.request_repaint();
            }
            if self.running {
                self.debug = String::from("running");
            } else if self.layout_time == 0.0 {
                self.layout_time = self.timer.elapsed().as_secs_f64();
                self.debug = String::from("done");
            }

            if let Some(cursor) = ctx.input(|i| i.pointer.hover_pos()) {
                self.cursor_loc = cursor.to_vec2();
            }
        });
        if !self.running {
            let pointer = ctx.input(|i| i.pointer.clone());
            if pointer.any_down() {
                if let Some(current_pos) = pointer.interact_pos() {
                    if !self.is_dragging {
                        self.is_dragging = true;
                        self.last_drag_pos = Some(current_pos);
                    } else if let Some(_last_pos) = self.last_drag_pos {
                        // let delta = current_pos - last_pos;

                        self.last_drag_pos = Some(current_pos);
                    }
                }
            } else {
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
        if let Some(cursor) = ctx.input(|i| i.pointer.hover_pos()) {
            self.cursor_loc = cursor.to_vec2();
        }
    }
}

fn main() {
    let graph = lockbookdata();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    thread::spawn(move || {
        let mut seconds = 0;
        while !stop_flag_clone.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            seconds += 1;
            println!("{} second", seconds);
        }
    });

    let graph = fix_graph(graph);

    let mut count: usize = 0;
    for node in &graph {
        println!("count is { }   node id is { }", node.id, count);
        count += 1;
    }
    let mut app = KnowledgeGraphApp::new(graph);
    app.build_directional_links();
    println!("{:?}", app.directional_links);
    app.bidiretional();
    app.label_clusters();
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

fn fix_graph(mut graph: Vec<LinkNode>) -> Vec<LinkNode> {
    graph.sort_by_key(|node| node.id);
    graph
}

fn draw_arrow(
    painter: &Painter,
    from: Pos2,
    to: Pos2,
    color: Color32,
    zoom_factor: f32,
    size: f32,
) {
    let x = from.x - to.x;
    let y = from.y - to.y;
    let angle = (y / x).atan();
    let new_x = size * angle.cos();
    let new_y: f32 = size * angle.sin();
    let intersect = Pos2::new(new_x, new_y);
    let to = to + intersect.to_vec2();
    let arrow_length = 6.0 * zoom_factor;
    let arrow_width = 4.0 * zoom_factor;

    let arrow_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 150);

    let direction = to - from;
    let distance = direction.length();

    if distance == 0.0 {
        return;
    }

    let dir = direction / distance;

    let arrow_base = to - dir * arrow_length;

    let perp = Vec2::new(-dir.y, dir.x);

    let arrow_p1 = arrow_base + perp * (arrow_width / 2.0);
    let arrow_p2 = arrow_base - perp * (arrow_width / 2.0);

    painter.line_segment(
        [from, arrow_base],
        Stroke::new(1.0 * zoom_factor, arrow_color),
    );

    let points = vec![to, arrow_p1, arrow_p2];

    painter.add(Shape::convex_polygon(
        points,
        arrow_color,
        Stroke::new(0.0, Color32::TRANSPARENT),
    ));
}

fn cursorin(cursor: Vec2, center: Pos2, size: f32) -> bool {
    if cursor.x > (center.x - size) && (center.x + size) > cursor.x {
        if cursor.y > (center.y - size) && (center.y + size) > cursor.y {
            return true;
        }
    }
    false
}
