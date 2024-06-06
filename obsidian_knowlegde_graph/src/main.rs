use eframe::egui;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f32;

#[derive(Serialize, Deserialize, Debug)]
struct LinkNode {
    id: usize,         // The unique id per node
    title: String,     // The name of the node
    links: Vec<usize>, // A vec of all the links
}

type Graph = Vec<LinkNode>; // Creates the graph data type

#[derive(Default)]
struct KnowledgeGraphApp {
    graph: Graph,                 // Stores the whole entire graph
    positions: Vec<egui::Pos2>, // Stores all the positions for rendering Pos2 is a position using x and y
    forces: Vec<egui::Vec2>,    // Stores the forces applied to each node
    zoom_factor: f32,           // Stores the zoom factor
    offset: egui::Vec2,         // Stores the offset for centering the zoom
    last_screen_size: egui::Vec2, // Stores the last screen size for maintaining positions
}

impl KnowledgeGraphApp {
    fn new(graph: Graph) -> Self {
        // Initializes the new method
        let positions = vec![egui::Pos2::ZERO; graph.len()]; // Makes all the positions (0,0)
        let forces = vec![egui::Vec2::ZERO; graph.len()]; // Initializes forces to zero
        Self {
            graph,
            positions,
            forces,
            zoom_factor: 1.0,
            offset: egui::Vec2::ZERO,
            last_screen_size: egui::Vec2::new(800.0, 600.0), // Initial screen size
        } // Returns a new KnowledgeGraph
    }
    fn apply_spring_layout(&mut self) {
        let width = 800.0; // Sets the width of the layout area
        let height = 600.0; // Sets the height of the layout area

        // Initial random placement within the bounds
        let mut rng = rand::thread_rng();
        self.positions = self
            .graph
            .iter()
            .map(|_| {
                egui::Pos2::new(
                    rng.gen_range(100.0..width - 100.0),
                    rng.gen_range(100.0..height - 100.0),
                )
            })
            .collect();

        let iterations = 5000; // Increased iterations
        let k = (width * height / (self.graph.len() as f32)).sqrt() * 0.175; // Adjusted spring constant
        let c = 0.005; // Smaller step size

        for _ in 0..iterations {
            // Reset forces
            for i in 0..self.graph.len() {
                self.forces[i] = egui::Vec2::ZERO;
            }

            // Calculate repulsive forces
            for i in 0..self.graph.len() {
                for j in 0..self.graph.len() {
                    if i != j {
                        let delta = self.positions[i] - self.positions[j];
                        let distance = delta.length().max(0.01); // Avoid division by zero
                        let repulsive_force = (k * k) / distance * 1.5; // Increased multiplier for stronger repulsive force
                        self.forces[i] += delta.normalized() * repulsive_force;
                    }
                }
            }

            // Calculate attractive forces
            for node in &self.graph {
                for &link in &node.links {
                    let delta = self.positions[node.id] - self.positions[link];
                    let distance = delta.length().max(0.01); // Avoid division by zero
                    let attractive_force = (distance * distance) / k * 0.5;
                    self.forces[node.id] -= delta.normalized() * attractive_force;
                    self.forces[link] += delta.normalized() * attractive_force;
                }
            }

            // Update positions based on forces
            for i in 0..self.graph.len() {
                self.positions[i] += self.forces[i] * c;

                // Ensure nodes do not overlap
                for j in 0..self.graph.len() {
                    if i != j {
                        let delta = self.positions[i] - self.positions[j];
                        let distance = delta.length().max(0.01); // Avoid division by zero
                        let min_distance = 30.0; // Minimum distance between nodes to prevent overlap

                        if distance < min_distance {
                            let overlap_force = (min_distance - distance) * 0.5; // Adjust overlap force
                            self.forces[i] += delta.normalized() * overlap_force;
                            self.forces[j] -= delta.normalized() * overlap_force;
                        }
                    }
                }

                // Ensure nodes are not too close to links
                for node in &self.graph {
                    for &link in &node.links {
                        if node.id != link {
                            let delta = self.positions[node.id] - self.positions[link];
                            let distance = delta.length().max(0.01); // Avoid division by zero
                            let link_distance = 20.0; // Minimum distance to maintain from the link

                            if distance < link_distance {
                                let link_force = (link_distance - distance) * 0.5; // Adjust link force
                                self.forces[node.id] += delta.normalized() * link_force;
                                self.forces[link] -= delta.normalized() * link_force;
                            }
                        }
                    }
                }

                // Keep nodes within bounds
                self.positions[i].x = self.positions[i].x.clamp(50.0, width - 50.0);
                self.positions[i].y = self.positions[i].y.clamp(50.0, height - 50.0);
            }
        }
    }

    fn draw_graph(&mut self, ui: &mut egui::Ui, screen_size: egui::Vec2) {
        // Center the zoom
        let center = screen_size / 2.0;

        // Calculate scale factors based on screen size change
        let scale_x = screen_size.x / self.last_screen_size.x;
        let scale_y = screen_size.y / self.last_screen_size.y;

        // Calculate the radius based on the number of nodes
        let radius = (30.0 * self.zoom_factor) / ((self.graph.len() as f32).sqrt() / 2.0).max(1.0);

        // Iterate over the nodes and draw them
        self.graph.iter().enumerate().for_each(|(i, node)| {
            // Adjust position based on scale and zoom
            let pos = center
                + ((self.positions[i].to_vec2() - center) * self.zoom_factor + self.offset)
                    * egui::Vec2::new(scale_x, scale_y);

            // Draw the links to connected nodes
            node.links
                .iter()
                .filter_map(|&link| {
                    self.positions.get(link).map(|&target_pos| {
                        let target_pos = center
                            + ((target_pos.to_vec2() - center) * self.zoom_factor + self.offset)
                                * egui::Vec2::new(scale_x, scale_y);
                        ui.painter().line_segment(
                            [pos.to_pos2(), target_pos.to_pos2()],
                            egui::Stroke::new(1.0 * self.zoom_factor, egui::Color32::GRAY),
                        );
                    })
                })
                .for_each(drop);

            // Draw the node circle
            ui.painter().circle(
                pos.to_pos2(),
                radius,
                egui::Color32::from_rgb(0, 0, 255),
                egui::Stroke::new(2.0 * self.zoom_factor, egui::Color32::BLACK),
            );

            // Draw the node title
            let font_id = egui::FontId::proportional(radius);
            ui.painter().text(
                pos.to_pos2(),
                egui::Align2::CENTER_CENTER,
                &node.title,
                font_id,
                egui::Color32::WHITE,
            );
        });

        // Draw the center point
        ui.painter().circle_filled(
            center.to_pos2(),
            5.0,                                // radius of the red dot
            egui::Color32::from_rgb(255, 0, 0), // red color
        );

        // Update the last screen size
        self.last_screen_size = screen_size;
    }
}

impl eframe::App for KnowledgeGraphApp {
    // Method called when eframe::run_native
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Any time anything happens it runs through this method
        egui::CentralPanel::default().show(ctx, |ui| {
            // Creates a panel for main content to be displayed
            ui.heading("Knowledge Graph");

            if ui.button("Apply Layout").clicked() {
                self.apply_spring_layout();
            }

            let screen_size = ui.available_size();
            self.draw_graph(ui, screen_size);
        });

        // Handle key press events for zoom
        if ctx.input(|i| i.key_pressed(egui::Key::Equals)) {
            self.zoom_factor *= 1.1; // Zoom in
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Minus)) {
            self.zoom_factor *= 0.9; // Zoom out
        }

        // Adjust offset to keep the view centered
        if ctx.input(|i| i.key_pressed(egui::Key::Equals))
            || ctx.input(|i| i.key_pressed(egui::Key::Minus))
        {
            let center = egui::Vec2::new(
                ctx.available_rect().width() / 2.0,
                ctx.available_rect().height() / 2.0,
            );
            self.offset = center * (1.0 - self.zoom_factor);
        }
    }
}

fn main() {
    // creates all the nodes for now
    let graph = vec![
        LinkNode {
            id: 0,
            title: String::from("Node 0"),
            links: vec![1, 2, 3, 4, 5, 6],
        },
        LinkNode {
            id: 1,
            title: String::from("Node 1"),
            links: vec![0, 7, 8],
        },
        LinkNode {
            id: 2,
            title: String::from("Node 2"),
            links: vec![0, 9],
        },
        LinkNode {
            id: 3,
            title: String::from("Node 3"),
            links: vec![0],
        },
        LinkNode {
            id: 4,
            title: String::from("Node 4"),
            links: vec![0, 10, 11, 12],
        },
        LinkNode {
            id: 5,
            title: String::from("Node 5"),
            links: vec![0],
        },
        LinkNode {
            id: 6,
            title: String::from("Node 6"),
            links: vec![0, 13, 14],
        },
        LinkNode {
            id: 7,
            title: String::from("Node 7"),
            links: vec![1, 15, 16],
        },
        LinkNode {
            id: 8,
            title: String::from("Node 8"),
            links: vec![1, 17],
        },
        LinkNode {
            id: 9,
            title: String::from("Node 9"),
            links: vec![2, 18, 19, 20],
        },
        LinkNode {
            id: 10,
            title: String::from("Node 10"),
            links: vec![4],
        },
        LinkNode {
            id: 11,
            title: String::from("Node 11"),
            links: vec![4, 21, 22],
        },
        LinkNode {
            id: 12,
            title: String::from("Node 12"),
            links: vec![4],
        },
        LinkNode {
            id: 13,
            title: String::from("Node 13"),
            links: vec![6, 23],
        },
        LinkNode {
            id: 14,
            title: String::from("Node 14"),
            links: vec![6, 24],
        },
        LinkNode {
            id: 15,
            title: String::from("Node 15"),
            links: vec![7],
        },
        LinkNode {
            id: 16,
            title: String::from("Node 16"),
            links: vec![7, 25, 26],
        },
        LinkNode {
            id: 17,
            title: String::from("Node 17"),
            links: vec![8, 27],
        },
        LinkNode {
            id: 18,
            title: String::from("Node 18"),
            links: vec![9],
        },
        LinkNode {
            id: 19,
            title: String::from("Node 19"),
            links: vec![9, 28],
        },
        LinkNode {
            id: 20,
            title: String::from("Node 20"),
            links: vec![9],
        },
        LinkNode {
            id: 21,
            title: String::from("Node 21"),
            links: vec![11],
        },
        LinkNode {
            id: 22,
            title: String::from("Node 22"),
            links: vec![11],
        },
        LinkNode {
            id: 23,
            title: String::from("Node 23"),
            links: vec![13],
        },
        LinkNode {
            id: 24,
            title: String::from("Node 24"),
            links: vec![14],
        },
        LinkNode {
            id: 25,
            title: String::from("Node 25"),
            links: vec![16],
        },
        LinkNode {
            id: 26,
            title: String::from("Node 26"),
            links: vec![16],
        },
        LinkNode {
            id: 27,
            title: String::from("Node 27"),
            links: vec![17],
        },
        LinkNode {
            id: 28,
            title: String::from("Node 28"),
            links: vec![19],
        },
        LinkNode {
            id: 29,
            title: String::from("Node 29"),
            links: vec![],
        },
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
