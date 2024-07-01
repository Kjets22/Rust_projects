use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::HashMap;

struct Note {
    id: usize,
    title: String,
    content: String,
    links: Vec<usize>,
}

struct KnowledgeGraph {
    notes: HashMap<usize, Note>,
    next_id: usize,
}

impl KnowledgeGraph {
    fn new() -> Self {
        KnowledgeGraph {
            notes: HashMap::new(),
            next_id: 0,
        }
    }

    fn add_note(&mut self, title: String, content: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let note = Note {
            id,
            title,
            content,
            links: Vec::new(),
        };
        self.notes.insert(id, note);
        id
    }

    fn add_link(&mut self, from_id: usize, to_id: usize) {
        if let Some(note) = self.notes.get_mut(&from_id) {
            if !note.links.contains(&to_id) {
                note.links.push(to_id);
            }
        }
    }
}

struct App {
    graph: KnowledgeGraph,
    selected_note: Option<usize>,
}

impl App {
    fn new() -> Self {
        let mut graph = KnowledgeGraph::new();
        let note1 = graph.add_note(
            "First Note".to_string(),
            "This is the content of the first note.".to_string(),
        );
        let note2 = graph.add_note(
            "Second Note".to_string(),
            "This is the content of the second note.".to_string(),
        );
        graph.add_link(note1, note2);

        App {
            graph,
            selected_note: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Knowledge Graph");

            // Draw the graph
            let graph_response = ui.allocate_response(ui.available_size(), egui::Sense::click());
            let rect = graph_response.rect;

            let painter = ui.painter();

            // Draw nodes
            for (id, note) in &self.graph.notes {
                let pos = Pos2::new(
                    rect.min.x + rect.width() * (id % 3) as f32 / 3.0,
                    rect.min.y + rect.height() * (id / 3) as f32 / 3.0,
                );
                painter.circle(
                    pos,
                    20.0,
                    Color32::LIGHT_BLUE,
                    Stroke::new(1.0, Color32::BLACK),
                );
                painter.text(
                    pos,
                    egui::Align2::CENTER_CENTER,
                    &note.title,
                    egui::TextStyle::Body.resolve(ui.style()),
                    Color32::BLACK,
                );
            }

            // Draw edges
            for (id, note) in &self.graph.notes {
                let from_pos = Pos2::new(
                    rect.min.x + rect.width() * (id % 3) as f32 / 3.0,
                    rect.min.y + rect.height() * (id / 3) as f32 / 3.0,
                );
                for &to_id in &note.links {
                    let to_pos = Pos2::new(
                        rect.min.x + rect.width() * (to_id % 3) as f32 / 3.0,
                        rect.min.y + rect.height() * (to_id / 3) as f32 / 3.0,
                    );
                    painter.line_segment([from_pos, to_pos], Stroke::new(1.0, Color32::GRAY));
                }
            }

            // Handle click on nodes
            if graph_response.clicked() {
                let click_pos = graph_response.interact_pointer_pos().unwrap();
                for (id, note) in &self.graph.notes {
                    let pos = Pos2::new(
                        rect.min.x + rect.width() * (id % 3) as f32 / 3.0,
                        rect.min.y + rect.height() * (id / 3) as f32 / 3.0,
                    );
                    if (click_pos - pos).length() < 20.0 {
                        self.selected_note = Some(*id);
                        break;
                    }
                }
            }

            // Show selected note content
            if let Some(id) = self.selected_note {
                if let Some(note) = self.graph.notes.get(&id) {
                    ui.separator();
                    ui.heading(&note.title);
                    ui.label(&note.content);
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Knowledge Graph",
        native_options,
        Box::new(|_cc| Box::new(App::new())),
    )
}
