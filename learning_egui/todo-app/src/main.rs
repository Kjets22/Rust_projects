use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
// Automatically implements the Default trait for TodoApp,
// allowing us to create a default instance of the struct.
// by adding Serialize and Deserialize it makes it so that
// that TodoApp can store and extract data

//implenet sroll area
//debounce
//apened only log
//log compaction
#[derive(Default, Serialize, Deserialize)]
struct TodoApp {
    tasks: Vec<String>,
    new_task: String,
}

impl TodoApp {
    //function of TodoApp that saves and load from file
    fn save_to_file(&self, path: &str) {
        if let Ok(mut file) = File::create(path)
        //open or creates a file path and returns a result which
        //is ok it succesfull
        {
            if let Ok(data) = serde_json::to_string(self)
            //contverts the TodoApp data from to a Json string
            {
                let _ = file.write_all(data.as_bytes());
            }
        }
    }

    fn load_from_file(path: &str) -> Self {
        //
        //let mut data = String::new();
        //File::open(path).map(|f|f.)
        if let Ok(mut file) = File::open(path) {
            let mut data = String::new();
            if file.read_to_string(&mut data).is_ok() {
                if let Ok(app) = serde_json::from_str(&data) {
                    return app;
                }
            }
        }
        Self::default()
    }
}

//in this code we are using self to refer to the TodoApp struct, since we
//are implementing a struct using for we can call the struct using self

impl eframe::App for TodoApp {
    //Implements the App trait from the eframe crate for our TodoApp struct.
    //The App trait requires defining the update method.

    fn update(
        &mut self,           //A mutable reference to the TodoApp instance
        ctx: &egui::Context, //The UI context, which provides access to the UI components and rendering.
        _frame: &mut eframe::Frame, //The frame provides information about the window and its properties,
                                    // but _frame is unused in this example (hence the underscore)
    ) {
        egui::CentralPanel::default().show(ctx, |ui|
            //Creates a central panel using egui::CentralPanel. 
            //The show method takes the UI context and a closure that defines the UI layout. 
            //The ui parameter in the closure is used to add widgets to the panel.


            {

            ui.heading("To-Do List");//Adds a heading to the UI with the text "To-Do List".

            ui.horizontal(|ui|{
                //Creates a horizontal layout within the central panel.
                // The closure defines the widgets placed horizontally                                                    
                let response = ui.text_edit_singleline(&mut self.new_task);
                //Adds a single-line text input for entering new tasks.
                //It binds to the new_task field of the TodoApp struct.                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                //so when enter is pressed focus is lost from the bar, then you check if the eneter key is pressed
                {
                    if !self.new_task.trim().is_empty() {
                        self.tasks.push(self.new_task.trim().to_string());
                        self.new_task.clear();
                    }
                    response.request_focus();//gives the focus back to the textbar
                }
                if ui.button("Add").clicked() {
                    //Adds an "Add" button. The clicked method returns true if the button was clicked.
                    if !self.new_task.trim().is_empty() {//it puts the text into tasks
                        self.tasks.push(self.new_task.trim().to_string());
                        self.new_task.clear();
                    }
                }
            });

            ui.separator();//adds another line seperator

            if let Some(first_task) = self.tasks.first()
            //what this does is tests if there are string in tasks
            //then it takes the first string in tasks and assigns it to first task
                {
                ui.heading(first_task);         //makes it so the first task in a heading instead of text

                if ui.button("Task Done").clicked() {//creates a button and is called when clicked
                    self.tasks.remove(0);     //removes the first task in the list of task
                }
            } else {
                ui.label("No tasks available.");  //then there are no tasks in the list 
            }

            ui.separator();//adds a horizontal line seprator in the UI

            for task in self.tasks.iter().skip(1) {//it displays all the tasks 
                ui.horizontal(|ui| {
                    ui.label(task);
                });
            }
        });
        self.save_to_file("tasks.json");
    }

    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.save_to_file("tasks.json");
    }
}

fn main() {
    let app = TodoApp::load_from_file("tasks.json"); //creates a default todo struct
    let native_options = eframe::NativeOptions::default(); // create a native window
    let _ = eframe::run_native(
        //runs the aplication using all the infromation given,
        "Simple To-Do App", //name of app
        native_options,     // window information
        Box::new(|_cc| Box::new(app)),
        //A closure that returns a boxed instance of the TodoApp.
    );
}
