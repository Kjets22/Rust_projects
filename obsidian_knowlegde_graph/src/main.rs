use eframe::egui;
use rand::Rng; // this crate is for genterating random numbers
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct LinkNode {
    id: usize,         // the unquie id per node
    title: String,     // the name of the node
    links: Vec<usize>, // a vec of al the links
}

type Graph = Vec<LinkNode>; // creates the graph data type

#[derive(Default)]
struct KnowledgeGraphApp {
    graph: Graph,               // stores the whole entire graph
    positions: Vec<egui::Pos2>, // stores all the postions for rendering Pos2 is a postion using x and y
}

impl KnowledgeGraphApp {
    fn new(graph: Graph) -> Self {
        //initlizes the new method
        let positions = vec![egui::Pos2::ZERO; graph.len()]; // makes all the postion 0,0
        Self { graph, positions } // this returns a new Knowledge graph
    }

    fn apply_force_directed_layout(&mut self) {
        // creates new postions for the nodes
        let width = 800.0; // sets area of layouts
        let height = 600.0;
        let center = egui::Pos2::new(width / 2.0, height / 2.0); // center postion
        let mut rng = rand::thread_rng(); // creates a random number generator

        self.positions = self
            .graph // it calls the graph function which has all the linknodes
            .iter() // it then makes a iter or each node
            .map(|_| {
                // it then maps the value of x and y with the random number
                let x = center.x + rng.gen_range(-200.0..200.0);
                let y = center.y + rng.gen_range(-200.0..200.0);
                egui::Pos2::new(x, y) //creates a new Pos2 using x and y found
            })
            .collect(); // collects all the values needed after a stream to collect
    }

    fn draw_graph(&self, ui: &mut egui::Ui) {
        // allocate painter for the UI area
        // think is needed no sure
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            egui::Sense::hover(),
        );

        // iterate over the nodes and draw them
        self.graph.iter().enumerate().for_each(|(i, node)| {
            let pos = self.positions[i];
            let radius = 20.0;

            // draw the links to connected nodes
            node.links
                .iter()
                .filter_map(|&link| {
                    self.positions.get(link).map(|&target_pos| {
                        painter.line_segment(
                            [pos, target_pos],
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                        );
                    })
                })
                .for_each(drop);
            // draw the node rectangle
            painter.circle(
                pos,
                radius,
                egui::Color32::from_rgb(0, 0, 255),
                egui::Stroke::new(2.0, egui::Color32::BLACK),
            );

            // draw the node title
            let font_id = egui::FontId::proportional(18.0);
            painter.text(
                pos,
                egui::Align2::CENTER_CENTER,
                &node.title,
                font_id,
                egui::Color32::WHITE,
            );
        });
    }
}

impl eframe::App for KnowledgeGraphApp {
    // method called when eframe::run_native
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // any time anything happens it runs through the is method
        egui::CentralPanel::default().show(ctx, |ui| {
            // creates a panel for main content to be displayed
            ui.heading("Knowledge Graph");

            if ui.button("Apply Layout").clicked() {
                self.apply_force_directed_layout();
            }

            self.draw_graph(ui);
        });
    }
}

fn main() {
    // creates all the nodes for now
    let graph = vec![
        LinkNode {
            id: 0,
            title: String::from("Node 0"),
            links: vec![1, 2],
        },
        LinkNode {
            id: 1,
            title: String::from("Node 1"),
            links: vec![0, 2],
        },
        LinkNode {
            id: 2,
            title: String::from("Node 2"),
            links: vec![0, 1],
        },
    ];

    let mut app = KnowledgeGraphApp::new(graph); // intilzes the knowledge map
    app.apply_force_directed_layout(); // apply layout initially
    let native_options = eframe::NativeOptions::default(); // creates a native window
    eframe::run_native(
        "Knowledge Graph App",
        native_options,
        Box::new(|_cc| Box::new(app)),
    );
}
