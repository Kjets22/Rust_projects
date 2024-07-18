use std::env;

use lb_rs::Core;
use serde::{Deserialize, Serialize};

pub type Graph = Vec<LinkNode>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkNode {
    pub id: usize,
    pub title: String,
    pub links: Vec<usize>,
    pub color: [f32; 3],
}

impl LinkNode {
    fn new(id: usize, title: String, links: Vec<usize>) -> Self {
        LinkNode {
            id,
            title,
            links,
            color: [0.0, 0.0, 0.0],
        }
    }
}

pub(crate) fn data() -> Graph {
    let core = core();
    for file in core.list_metadatas().unwrap() {
        if file.is_document() && file.name.ends_with(".md") {
            let doc = core.read_document(file.id).unwrap();
            let doc = String::from_utf8(doc).unwrap();

            // todo for krish
            // add a function that detects links in strings
            // level 1 complexity -- use a regex or crate to detect strings (ask travis in
            // engineering "how do I detect a string"
            // level 2 complexity -- handle the 3 types of links in your data model
            // raw dog google.com, markdown link []() to an external site, md link to within
            // lockbook (lb://file-id), md link that's relative (../todo.md).
            // parth will author some docuemntation about all the links types, ask me to do that
            // level 3 complexity -- given a destination how do you label it. for lockbook
            // documents file name is good, for external sites, what portion of the URL do you hang
            // on to? https://parth.cafe/, or https://parth.cafe/p/creating-a-sick-cli? How do you
            // label these? (use the title?) imo as a first parth.cafe
            // consider weighting the size of the node based on back references
            // consider an algorithm for data generation as well as data visualization that is
            // incremental
            println!("{doc}");
        }
    }
    todo!()
}

fn core() -> Core {
    let writeable_path = writable_path();

    Core::init(&lb_rs::Config {
        writeable_path,
        logs: true,
        colored_logs: true,
    })
    .unwrap()
}

fn writable_path() -> String {
    let specified_path = env::var("LOCKBOOK_PATH");

    let default_path = env::var("HOME") // unix
        .or(env::var("HOMEPATH")) // windows
        .map(|home| format!("{home}/.lockbook/cli"));

    specified_path.or(default_path).unwrap()
}
