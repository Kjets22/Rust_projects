use std::env;

use egui::Checkbox;
use lb_rs::{Core, File};
use regex::Regex;
use serde::{Deserialize, Serialize};

pub type Graph = Vec<LinkNode>;

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkNode {
    pub id: usize,
    pub title: String,
    pub links: Vec<usize>,
    pub color: [f32; 3],
}

#[derive(Clone)]
pub struct Name_Id {
    pub id: usize,
    pub name: String,
}

impl Name_Id {
    fn new(id: usize, name: String) -> Self {
        Name_Id { id, name }
    }
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
    vec![
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
    ]
}

pub(crate) fn lockbookdata() -> Graph {
    let mut graph: Graph = Vec::new();
    let mut classify: Vec<Name_Id> = Vec::new();
    let core = core();
    let mut id: usize = 1;
    let mut num_linsk = 1;
    for file in core.list_metadatas().unwrap() {
        if file.is_document() && file.name.ends_with(".md") {
            let doc = core.read_document(file.id).unwrap();
            let doc = String::from_utf8(doc).unwrap();
            let name = file.name;
            let links = checkforlinks(&mut classify, &mut id, &doc);
            num_linsk += links.len();
            graph.push(LinkNode::new(in_classify(&name, &classify), name, links));
            //let node_id=
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
            //println!("{doc}");
            //println!("{:?}", core.list_metadatas());
            //println!("{:?}", core.get_account().unwrap());
        }
    }
    for item in classify.iter() {
        let mut found = false;
        let name = &item.name;
        for link in &graph {
            if name == &link.title {
                found = true;
            }
        }
        if !found {
            graph.push(LinkNode::new(
                in_classify(&item.name, &classify),
                name.to_string(),
                vec![],
            ));
        }
    }
    println!("{}", id);
    println!("{}", num_linsk);

    graph
}
// every time you finda  link and check it is in the classify if not make a new one
// also make classify a hasmap so its easyier to search with a counter for the id so it knows what id to put
// nm don't make it a haskmap it will make it much harder

// one probleom that will need to fixed files can have same name if in diffrent folders
// plan
// make it so it is a for loop where when it finds a link it then goes to the link and class checklink
// onn that data then for every link found i want it to call the springlayout function but with a delay of 0
// so you see it get all the links should look nice
fn checkforlinks(classify: &mut Vec<Name_Id>, id: &mut usize, doc: &str) -> Vec<usize> {
    let mut links: Vec<usize> = Vec::new();
    let link_names = find_links(doc);
    //find all links and
    for link in link_names {
        let link_id = in_classify(&link, &classify);
        if link_id == 0 {
            classify.push(Name_Id::new(*id, link));
            links.push(*id);
            *id += 1;
        } else {
            links.push(link_id);
        }
    }
    links
}

fn find_links(text: &str) -> Vec<String> {
    // Regex pattern to match most common types of URLs
    let url_pattern = r"(https?://|lb:)[^\s/$.?#].[^\s]*";
    let re = Regex::new(url_pattern).unwrap();

    // Collect all the matches into a Vec<String>
    let links: Vec<String> = re
        .find_iter(text)
        .map(|mat| mat.as_str().to_string())
        .collect();

    // Print each link found
    for link in &links {
        println!("Found link: {}", link);
    }

    // Return the links
    links
}

fn in_classify(name: &String, classify: &Vec<Name_Id>) -> usize {
    let mut id: usize = 0;
    for linkinfo in classify {
        if &linkinfo.name == name {
            id = linkinfo.id;
        }
    }
    id
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
