use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{f32, fmt::Debug, time::Instant};

#[derive(Serialize, Deserialize, Debug)]
struct LinkNode {
    id: usize,
    title: String,
    links: Vec<usize>,
    color: [i32; 3],
}

impl LinkNode {
    fn new(id: usize, title: String, links: Vec<usize>) -> Self {
        LinkNode {
            id,
            title,
            links,
            color: [0, 0, 255],
        }
    }
}
type Graph = Vec<LinkNode>;

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
    layout_time: f64, // New field to store layout computation time
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
            layout_time: 0.0, // Initialize layout time
        }
    }

    fn apply_spring_layout(&mut self) {
        let start_time = Instant::now(); // Start timing

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

        let iterations = 200 * self.graph.len();
        let k = (width * height / (self.graph.len() as f32)).sqrt() * 0.2;
        let c = 0.005;

        for _ in 0..iterations {
            for i in 0..self.graph.len() {
                self.forces[i] = egui::Vec2::ZERO;
            }

            for i in 0..self.graph.len() {
                for j in 0..self.graph.len() {
                    if i != j {
                        let delta = self.positions[i] - self.positions[j];
                        let distance = delta.length().max(0.01);
                        let repulsive_force = (k * k) / distance * (10.0 / (num_nodes * num_nodes));
                        self.forces[i] += delta.normalized() * repulsive_force;
                    }
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
                for j in 0..self.graph.len() {
                    if i != j {
                        let delta = self.positions[i] - self.positions[j];
                        let distance = delta.length().max(0.01);
                        let min_distance = 100.0;

                        if distance < min_distance {
                            let overlap_force = (min_distance - distance) * 0.5;
                            self.forces[i] += delta.normalized() * overlap_force;
                            self.forces[j] -= delta.normalized() * overlap_force;
                        }
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

                // Keep nodes within bounds (commented out as in the original)
                // self.positions[i].x = self.positions[i].x.max(50.0).min(width - 50.0);
                // self.positions[i].y = self.positions[i].y.max(50.0).min(height - 50.0);
            }
        }

        let end_time = Instant::now(); // End timing
        self.layout_time = (end_time - start_time).as_secs_f64(); // Store the elapsed time
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui, screen_size: egui::Vec2) {
        let center = screen_size / 2.0;
        let scale_x = screen_size.x / self.last_screen_size.x;
        let scale_y = screen_size.y / self.last_screen_size.y;
        let radius = (30.0 * self.zoom_factor) / ((self.graph.len() as f32).sqrt() / 3.0).max(1.0);

        self.graph.iter().enumerate().for_each(|(i, node)| {
            let pos = self.positions[i].to_vec2();

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

            ui.painter().circle(
                pos.to_pos2(),
                radius,
                egui::Color32::from_rgb(0, 0, 255),
                egui::Stroke::new(2.0 * self.zoom_factor, egui::Color32::BLACK),
            );

            let font_id = egui::FontId::proportional(radius);
            ui.painter().text(
                pos.to_pos2(),
                egui::Align2::CENTER_CENTER,
                &node.title,
                font_id,
                egui::Color32::WHITE,
            );
        });

        ui.painter()
            .circle_filled(center.to_pos2(), 5.0, egui::Color32::from_rgb(255, 0, 0));

        // Display the layout computation time
        ui.label(format!(
            "Layout computation time: {:.2} seconds",
            self.layout_time
        ));

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
}

impl eframe::App for KnowledgeGraphApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Knowledge Graph");
            ui.text_edit_singleline(&mut self.debug);

            // Display the layout computation time
            ui.label(format!(
                "Layout computation time: {:.2} seconds",
                self.layout_time
            ));

            let screen_size = ui.available_size();

            self.draw_graph(ui, screen_size);
        });

        let pointer = ctx.input(|i| i.pointer.clone());
        if pointer.any_down() {
            if let Some(current_pos) = pointer.interact_pos() {
                if !self.is_dragging {
                    self.is_dragging = true;
                    self.last_drag_pos = Some(current_pos);
                    self.debug = String::from("clicked");
                } else if let Some(last_pos) = self.last_drag_pos {
                    self.debug = String::from("dragged");
                    let delta = current_pos - last_pos;
                    self.draged(delta);
                    self.last_drag_pos = Some(current_pos);
                }
            }
        } else {
            self.debug = String::from("clicked");
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
    // creates all the nodes for now
    let graph = vec![
        // Subgraph 1
        LinkNode::new(0, String::from("Node 0"), vec![1, 2, 3, 4, 5]),
        LinkNode::new(1, String::from("Node 1"), vec![0, 6, 7, 8, 9]),
        LinkNode::new(2, String::from("Node 2"), vec![0, 10, 11, 12]),
        LinkNode::new(3, String::from("Node 3"), vec![0, 13, 14, 15, 16]),
        LinkNode::new(4, String::from("Node 4"), vec![0, 17, 18]),
        LinkNode::new(5, String::from("Node 5"), vec![0, 19, 20, 21]),
        LinkNode::new(6, String::from("Node 6"), vec![1, 22, 23, 24, 25]),
        LinkNode::new(7, String::from("Node 7"), vec![1, 26, 27]),
        LinkNode::new(8, String::from("Node 8"), vec![1, 28, 29, 30, 31]),
        LinkNode::new(9, String::from("Node 9"), vec![1, 32, 33]),
        LinkNode::new(10, String::from("Node 10"), vec![2, 34, 35]),
        LinkNode::new(11, String::from("Node 11"), vec![2, 36, 37, 38]),
        LinkNode::new(12, String::from("Node 12"), vec![2, 39]),
        LinkNode::new(13, String::from("Node 13"), vec![3, 40, 41]),
        LinkNode::new(14, String::from("Node 14"), vec![3, 42, 43, 44, 45]),
        LinkNode::new(15, String::from("Node 15"), vec![3, 46, 47]),
        LinkNode::new(16, String::from("Node 16"), vec![3, 48, 49]),
        LinkNode::new(17, String::from("Node 17"), vec![4]),
        LinkNode::new(18, String::from("Node 18"), vec![4]),
        LinkNode::new(19, String::from("Node 19"), vec![5]),
        LinkNode::new(20, String::from("Node 20"), vec![5]),
        LinkNode::new(21, String::from("Node 21"), vec![5]),
        LinkNode::new(22, String::from("Node 22"), vec![6]),
        LinkNode::new(23, String::from("Node 23"), vec![6]),
        LinkNode::new(24, String::from("Node 24"), vec![6]),
        LinkNode::new(25, String::from("Node 25"), vec![6]),
        LinkNode::new(26, String::from("Node 26"), vec![7]),
        LinkNode::new(27, String::from("Node 27"), vec![7]),
        LinkNode::new(28, String::from("Node 28"), vec![8]),
        LinkNode::new(29, String::from("Node 29"), vec![8]),
        LinkNode::new(30, String::from("Node 30"), vec![8]),
        LinkNode::new(31, String::from("Node 31"), vec![8]),
        LinkNode::new(32, String::from("Node 32"), vec![9]),
        LinkNode::new(33, String::from("Node 33"), vec![9]),
        LinkNode::new(34, String::from("Node 34"), vec![10]),
        LinkNode::new(35, String::from("Node 35"), vec![10]),
        LinkNode::new(36, String::from("Node 36"), vec![11]),
        LinkNode::new(37, String::from("Node 37"), vec![11]),
        LinkNode::new(38, String::from("Node 38"), vec![11]),
        LinkNode::new(39, String::from("Node 39"), vec![12]),
        LinkNode::new(40, String::from("Node 40"), vec![13]),
        LinkNode::new(41, String::from("Node 41"), vec![13]),
        LinkNode::new(42, String::from("Node 42"), vec![14]),
        LinkNode::new(43, String::from("Node 43"), vec![14]),
        LinkNode::new(44, String::from("Node 44"), vec![14]),
        LinkNode::new(45, String::from("Node 45"), vec![14]),
        LinkNode::new(46, String::from("Node 46"), vec![15]),
        LinkNode::new(47, String::from("Node 47"), vec![15]),
        LinkNode::new(48, String::from("Node 48"), vec![16]),
        LinkNode::new(49, String::from("Node 49"), vec![16]),
        LinkNode::new(50, String::from("Node 50"), vec![51, 52, 53, 54, 55]),
        LinkNode::new(51, String::from("Node 51"), vec![50, 56, 57, 58, 59]),
        LinkNode::new(52, String::from("Node 52"), vec![50, 60, 61, 62, 63]),
        LinkNode::new(53, String::from("Node 53"), vec![50, 64, 65, 66, 67]),
        LinkNode::new(54, String::from("Node 54"), vec![50, 68, 69, 70]),
        LinkNode::new(55, String::from("Node 55"), vec![50, 71, 72, 73]),
        LinkNode::new(56, String::from("Node 56"), vec![51, 74, 75, 76]),
        LinkNode::new(57, String::from("Node 57"), vec![51, 77, 78, 79]),
        LinkNode::new(58, String::from("Node 58"), vec![52, 80, 81, 82, 83]),
        LinkNode::new(59, String::from("Node 59"), vec![52, 84, 85]),
        LinkNode::new(60, String::from("Node 60"), vec![52, 86, 87]),
        LinkNode::new(61, String::from("Node 61"), vec![52, 88, 89]),
        LinkNode::new(62, String::from("Node 62"), vec![53, 90, 91, 92]),
        LinkNode::new(63, String::from("Node 63"), vec![53, 93, 94]),
        LinkNode::new(64, String::from("Node 64"), vec![53, 95, 96, 97]),
        LinkNode::new(65, String::from("Node 65"), vec![53, 98, 99]),
        LinkNode::new(66, String::from("Node 66"), vec![54, 99]),
        LinkNode::new(67, String::from("Node 67"), vec![54]),
        LinkNode::new(68, String::from("Node 68"), vec![54]),
        LinkNode::new(69, String::from("Node 69"), vec![54]),
        LinkNode::new(70, String::from("Node 70"), vec![54]),
        LinkNode::new(71, String::from("Node 71"), vec![55]),
        LinkNode::new(72, String::from("Node 72"), vec![55]),
        LinkNode::new(73, String::from("Node 73"), vec![55]),
        LinkNode::new(74, String::from("Node 74"), vec![56]),
        LinkNode::new(75, String::from("Node 75"), vec![56]),
        LinkNode::new(76, String::from("Node 76"), vec![56]),
        LinkNode::new(77, String::from("Node 77"), vec![57]),
        LinkNode::new(78, String::from("Node 78"), vec![57]),
        LinkNode::new(79, String::from("Node 79"), vec![57]),
        LinkNode::new(80, String::from("Node 80"), vec![58]),
        LinkNode::new(81, String::from("Node 81"), vec![58]),
        LinkNode::new(82, String::from("Node 82"), vec![58]),
        LinkNode::new(83, String::from("Node 83"), vec![58]),
        LinkNode::new(84, String::from("Node 84"), vec![59]),
        LinkNode::new(85, String::from("Node 85"), vec![59]),
        LinkNode::new(86, String::from("Node 86"), vec![60]),
        LinkNode::new(87, String::from("Node 87"), vec![60]),
        LinkNode::new(88, String::from("Node 88"), vec![61]),
        LinkNode::new(89, String::from("Node 89"), vec![61]),
        LinkNode::new(90, String::from("Node 90"), vec![62]),
        LinkNode::new(91, String::from("Node 91"), vec![62]),
        LinkNode::new(92, String::from("Node 92"), vec![62]),
        LinkNode::new(93, String::from("Node 93"), vec![63]),
        LinkNode::new(94, String::from("Node 94"), vec![63]),
        LinkNode::new(95, String::from("Node 95"), vec![64]),
        LinkNode::new(96, String::from("Node 96"), vec![64]),
        LinkNode::new(97, String::from("Node 97"), vec![64]),
        LinkNode::new(98, String::from("Node 98"), vec![65]),
        LinkNode::new(99, String::from("Node 99"), vec![65]),
    ];
    let mut app = KnowledgeGraphApp::new(graph); // intilzes the knowledge map
    app.apply_spring_layout(); // apply layout initially
    let native_options = eframe::NativeOptions::default(); // creates a native window
    eframe::run_native(
        "Knowledge Graph App",
        native_options,
        Box::new(|_cc| Box::new(app)),
    );
}
