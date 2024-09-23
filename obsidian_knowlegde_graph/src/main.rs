mod data;

use data::{data, lockbookdata, Graph};
use eframe::glow::{INFO_LOG_LENGTH, NOR};
use eframe::{egui, App, Frame};
use egui::emath::Numeric;
use egui::style::Selection;
use egui::Pos2;
use rand::Rng;

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{f32, time::Instant};
use std::{thread, usize};

use crate::data::LinkNode;

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

    /// Initializes the positions of all nodes in the graph based on their cluster assignments.
    /// - Clusters are arranged on a main circle with sufficient spacing to prevent overlap.
    /// - Nodes within each cluster are arranged in a small circle of radius 10.
    /// - Unlinked nodes (without a `cluster_id`) are placed in their own separate cluster inside the main circle.

    /// Initializes the positions of all nodes in the graph based on their cluster assignments.
    /// - Clusters with links are arranged inside a main circle without overlapping.
    /// - Each cluster's nodes are arranged in their own small circle with a radius of 10.
    /// - Unlinked nodes (without a `cluster_id`) are arranged on the outer perimeter of the main circle.

    fn initialize_positions(&mut self) {
        // Define the dimensions of the canvas
        let width = 800.0;
        let height = 600.0;

        // Define the center of the main circle
        let main_center = Pos2::new(width / 2.0, height / 2.0);

        // Define radii
        let cluster_small_radius = 10.0; // Radius for arranging nodes within a cluster
        let main_circle_radius = 300.0; // Radius of the main circle where multi-node clusters are placed
        let outer_circle_radius = main_circle_radius + 100.0; // Radius for single-node clusters and unlinked nodes
        let buffer = 5.0; // Additional buffer to prevent overlap between clusters

        let mut positions_map = HashMap::new();
        let mut clusters: HashMap<Option<usize>, Vec<usize>> = HashMap::new();
        let mut unlinked_nodes: Vec<usize> = Vec::new();

        // 1. Group nodes by cluster_id or mark as unlinked if cluster_id is None
        for node in &self.graph {
            if node.links.len() == 0 {
                unlinked_nodes.push(node.id)
            } else {
                clusters
                    .entry(node.cluster_id)
                    .or_insert_with(Vec::new)
                    .push(node.id);
            }
        }
        // 3. Arrange multi-node clusters on the main circle
        let num_multi_clusters = clusters.len();

        if num_multi_clusters > 0 {
            // Calculate the minimum angle between clusters to prevent overlap

            // Calculate the angle step between clusters to distribute them evenly
            let angle_step_clusters = 2.0 * std::f32::consts::PI / num_multi_clusters as f32;
            for (cluster_id, node_ids) in clusters {
                let min_distance: f32 = 2.0 * cluster_small_radius + buffer; // Minimum chord distance between cluster centers
                let min_angle = 2.0 * (min_distance / (2.0 * main_circle_radius)).asin(); // Minimum angle in radians

                // Total required angle for all multi-node clusters
                let total_required_angle = num_multi_clusters as f32 * min_angle;

                // Check if the total required angle exceeds the full circle
                if total_required_angle > 2.0 * std::f32::consts::PI {
                    eprintln!("Warning: Not enough space on the main circle to arrange all multi-node clusters without overlap.");
                    // Optionally, you can increase the main_circle_radius or reduce the number of clusters
                }

                let number_nodes = node_ids.len();
                let angle: f32 = cluster_id.unwrap() as f32 * angle_step_clusters;

                let cluster_center = Pos2::new(
                    main_center.x + main_circle_radius * angle.cos(),
                    main_center.y + main_circle_radius * angle.sin(),
                );
                let angle_step_nodes = 2.0 * std::f32::consts::PI / number_nodes as f32;
                let mut count: f32 = 0.0;

                for node_id in node_ids {
                    let node_angle = count * angle_step_nodes;
                    let node_pos = Pos2::new(
                        cluster_center.x
                            + number_nodes as f32
                            + cluster_small_radius * node_angle.cos(),
                        cluster_center.y
                            + number_nodes as f32
                            + cluster_small_radius * node_angle.sin(),
                    );
                    positions_map.insert(node_id, node_pos);
                    count += 1.0;
                }
            }
        }

        // 4. Arrange single-node clusters and unlinked nodes on the outer perimeter
        let total_outer_nodes = unlinked_nodes.len();

        if total_outer_nodes > 0 {
            let angle_step_outer = 2.0 * std::f32::consts::PI / total_outer_nodes as f32;

            for (i, &node_id) in unlinked_nodes.iter().enumerate() {
                let mut nocluster: Option<usize> = None;
                self.graph[node_id].cluster_id = nocluster;
                // let angle = i as f32 * angle_step_outer;
                // let node_pos = Pos2::new(
                //     main_center.x + outer_circle_radius * angle.cos(),
                //     main_center.y + outer_circle_radius * angle.sin(),
                // );
                // positions_map.insert(node_id, node_pos);
            }
        }

        // 5. Assign positions to all nodes, defaulting to the main center if missing
        self.positions = (0..self.graph.len())
            .map(|i| *positions_map.get(&i).unwrap_or(&main_center))
            .collect();
    }

    fn apply_spring_layout(&mut self) {
        let width = 700.0;
        let height = 500.0;
        let num_nodes = self.graph.len() as f32;

        // Define spring layout constants
        let k_spring = 0.01; // Spring constant for attraction
        let k_repel = 1500.0; // Repulsion constant between nodes
        let k_link_nudge = 0.005; // Small nudge constant for node-link interaction
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
                        let repulsive_force = k_repel / (distance * distance);
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
                let attractive_force = k_spring * (distance * distance);
                let attraction = delta.normalized() * attractive_force;

                // Apply the attraction between connected nodes
                self.forces[node.id] -= attraction;
                self.forces[link] += attraction;
            }
        }

        // ** New code start **
        // Apply weak random nudges to nodes close to link lines
        for node in &self.graph {
            let pos_node = self.positions[node.id];

            // Iterate over all edges (links)
            for other_node in &self.graph {
                for &link in &other_node.links {
                    // Skip if the node is part of the link
                    if node.id == other_node.id || node.id == link {
                        continue;
                    }

                    let pos_a = self.positions[other_node.id];
                    let pos_b = self.positions[link];

                    // Calculate the closest point on the line segment to the node
                    let line_vec = pos_b - pos_a;
                    let node_vec = pos_node - pos_a;

                    let line_length_sq = line_vec.dot(line_vec).max(0.01); // Avoid division by zero
                    let t = (node_vec.dot(line_vec) / line_length_sq).clamp(0.0, 1.0);
                    let closest_point = pos_a + line_vec * t;

                    let delta = pos_node - closest_point;
                    let distance = delta.length().max(0.01); // Avoid division by zero

                    // If the node is close to the link line, apply a weak random nudge
                    if distance < cell_size * 0.5 {
                        // Adjust the threshold as needed
                        // Generate a small random direction vector
                        let mut rng = rand::thread_rng();
                        let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
                        let random_direction = egui::Vec2::angled(angle);

                        // Scale the nudge to be weaker than other forces
                        let nudge_magnitude = k_link_nudge * distance;
                        let nudge = random_direction * nudge_magnitude;

                        // Apply the nudge to the node's force
                        self.forces[node.id] += nudge;
                    }
                }
            }
        }
        // ** New code end **

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
        }

        // Increment the iteration on every pass
        self.iteration += 1;

        // Check for convergence by monitoring the total change in position
        let total_change = self.forces.iter().map(|f| f.length()).sum::<f32>();

        if total_change < 20.0 || self.iteration >= 30000 {
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

        let base_size = radius;
        let k = 1.0; // Adjust this constant to control the scaling

        // First, draw all the links
        for (i, node) in self.graph.iter().enumerate() {
            if node.cluster_id.is_some() {
                let pos = self.positions[i].to_vec2();

                node.links
                    .iter()
                    .filter_map(|&link| {
                        if let Some(&target_pos) = self.positions.get(link) {
                            let target_pos = target_pos.to_vec2();
                            Some(ui.painter().line_segment(
                                [pos.to_pos2(), target_pos.to_pos2()],
                                egui::Stroke::new(1.0 * self.zoom_factor, egui::Color32::GRAY),
                            ))
                        } else {
                            None
                        }
                    })
                    .for_each(drop);
            }
        }

        // Then, draw all the nodes on top of the links
        for (i, node) in self.graph.iter().enumerate() {
            let rgb_color = egui::Color32::from_rgb(
                (node.color[0] * 255.0) as u8,
                (node.color[1] * 255.0) as u8,
                (node.color[2] * 255.0) as u8,
            );

            let n = node.links.len() as f32;
            let size = base_size + k * (n + 1.0).ln();

            if node.cluster_id.is_some() {
                let pos = self.positions[i].to_vec2();

                // Draw the node circle
                ui.painter().circle(
                    pos.to_pos2(),
                    size,
                    rgb_color,
                    egui::Stroke::new(2.0 * self.zoom_factor, egui::Color32::BLACK),
                );

                // Draw the node title if the radius is large enough
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
            }
        }

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
                if self.graph[i].links.is_empty() {
                    // If the node has no links, assign white color
                    self.graph[i].color = [1.0, 1.0, 1.0];
                } else {
                    self.dfs(i, bluecol, redcol, greencol);

                    // Update color values in a way that keeps them within a visible range
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
        } // Step 2: Iterate over the collected node IDs and perform clustering
        let mut count = 1;
        println!("{}", node_ids.len());
        for node_id in node_ids {
            println!("Node {}", node_id);
            self.clusters(node_id, count);
            count += 1;
        }
        println!("{:?}", self.graph);
    }

    /// Recursively assigns a cluster_id to the node and all connected nodes.

    fn clusters(&mut self, node_id: usize, cluster_id: usize) {
        // Check if the node is already assigned to a cluster
        if self.graph[node_id].cluster_id.is_some() {
            return; // Already assigned
        }

        // Assign the cluster_id to the current node
        self.graph[node_id].cluster_id = Some(cluster_id);
        println!("Node {} assigned to cluster {}", node_id, cluster_id);

        // Clone the links to avoid holding an immutable borrow during mutable borrow
        let links = self.graph[node_id].links.clone();

        // Recursively assign the same cluster_id to all linked nodes
        for link in links {
            if link != node_id {
                // Prevent self-loop infinite recursion
                // if(self.graph[link])
                self.clusters(link, cluster_id);
            }
        }
    }

    fn verify_and_assign_clusters(&mut self) {
        // Define radii
        let main_circle_radius = 200.0_f32; // Radius of the main circle where multi-node clusters are placed
        let outer_circle_radius = main_circle_radius + 100.0_f32; // Radius for single-node clusters and unlinked nodes
        let cluster_small_radius = 10.0_f32; // Radius for arranging nodes within a cluster

        // Iterate through all nodes to find those on the outer circle
        for (i, node) in self.graph.iter().enumerate() {
            // Calculate distance from main center
            let distance = self.positions[i].distance(Pos2::new(800.0_f32 / 2.0, 600.0_f32 / 2.0));

            // Define a tolerance for floating point comparison
            let tolerance = 1.0_f32;

            // Check if node is on the outer circle within the tolerance
            if (distance - outer_circle_radius).abs() < tolerance {
                if let Some(cluster_id) = node.cluster_id {
                    // Check if the cluster has more than one node
                    if let cluster_nodes = self.get_cluster_nodes(cluster_id) {
                        if cluster_nodes.len() > 1 {
                            // Calculate the angle to place the node within its cluster's small circle
                            let cluster_center = self.get_cluster_center(cluster_id);
                            let angle = (self.positions[i].y - cluster_center.y)
                                .atan2(self.positions[i].x - cluster_center.x);
                            let new_pos = Pos2::new(
                                cluster_center.x + cluster_small_radius * angle.cos(),
                                cluster_center.y + cluster_small_radius * angle.sin(),
                            );
                            self.positions[i] = new_pos;
                            println!("Node {} reassigned to its cluster {}", i, cluster_id);
                        }
                    }
                }
            }
        }
    }

    /// Helper function to get all nodes in a cluster by cluster_id

    fn get_cluster_nodes(&self, cluster_id: usize) -> Vec<usize> {
        self.graph
            .iter()
            .enumerate()
            .filter(|(_, node)| node.cluster_id == Some(cluster_id))
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>() // Return the vector directly, not a reference
    }
    /// Helper function to get the center position of a cluster by cluster_id
    fn get_cluster_center(&self, cluster_id: usize) -> Pos2 {
        // Assuming that the first node in the cluster determines the cluster center
        for (i, node) in self.graph.iter().enumerate() {
            if node.cluster_id == Some(cluster_id) {
                return self.positions[i];
            }
        }
        // Fallback to main center if not found
        Pos2::new(800.0_f32 / 2.0, 600.0_f32 / 2.0)
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
            // self.initialize_positions(); //rmb to comment this out with you uncommet spring layout
            // self.running = false;

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

    let len = graph.len();
    let graph = fix_graph(graph);

    let mut count: usize = 0;
    for node in &graph {
        println!("count is { }   node id is { }", node.id, count);
        count += 1;
    }
    let mut app = KnowledgeGraphApp::new(graph);
    app.label_clusters();
    app.label_subgraphs();
    //app.verify_and_assign_clusters();
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
    // Sort the graph in place by the `id` field
    graph.sort_by_key(|node| node.id);
    graph
}
// fn fix_graph(graph: Vec<LinkNode>, len: usize) -> Vec<LinkNode> {
//     let mut new_graph: Vec<LinkNode> = Vec::new();
//     let mut graph_id: Vec<(usize, usize)> = Vec::new();
//     let mut count: usize = 0;
//     for node in &graph {
//         graph_id.push((node.id, count));
//         count += 1;
//     }

//     for node_placement in 0..len {
//         for (node_id, postion) in graph_id {
//             if (node_id == node_placement) {
//                 new_graph.push(graph[postion]);
//             }
//         }
//     }
//     new_graph
// }
