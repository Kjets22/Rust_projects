use std::env;

use lb_rs::Core;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub type Graph = Vec<LinkNode>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkNode {
    pub id: usize,
    pub title: String,
    pub links: Vec<usize>,
    pub color: [f32; 3],
    pub cluster_id: Option<usize>,
    pub internal: bool,

    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub fx: Option<f32>,
    pub fy: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct Name_Id {
    pub id: usize,
    pub name: String,
    pub links: Vec<usize>,
    pub internal: bool,
}

impl Name_Id {
    fn new(id: usize, name: String, links: Vec<usize>) -> Self {
        Name_Id {
            id,
            name,
            links,
            internal: true,
        }
    }
}

impl LinkNode {
    fn new(id: usize, title: String, links_given: Vec<usize>) -> Self {
        LinkNode {
            id,
            title,
            links: links_given.clone(),
            color: [0.0, 0.0, 0.0],
            cluster_id: None,
            internal: true,

            x: 0.0, // Set to an initial value, e.g., random or based on index
            y: 0.0, // Set to an initial value
            vx: 0.0,
            vy: 0.0,
            fx: None,
            fy: None,
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
    let mut id: usize = 0;
    let mut num_links = 1;
    let mut info: Vec<(String, String)> = Vec::new();

    for file in core.list_metadatas().unwrap() {
        if file.is_document() && file.name.ends_with(".md") {
            let doc = core.read_document(file.id).unwrap();
            let doc = String::from_utf8(doc).unwrap();
            let name = file.name;
            info.push((name, doc));
            //classify.push(Name_Id::new(classify.len(), name.clone(), vec![]));
        }
    }

    info.sort_by(|a, b| a.0.cmp(&b.0));
    for n in info {
        // Check for links in the document
        let doc = n.1;
        let name = n.0;
        let links = checkforlinks(&mut classify, &mut id, &doc);
        id += 1;
        num_links += links.len();
        classify.push(Name_Id::new(classify.len(), name.clone(), links));
        //add_links(links, &mut getName_Id(&name, &classify));
        // println!("{:?}", getName_Id(&name, &classify));
        //getName_Id(&name, &classify).links = links.clone;

        // Add the document as a node with its links
        // graph.push(LinkNode::new(
        //     in_classify(&name, &classify).unwrap(),
        //     name.clone(),
        //     getName_Id(&name, &classify).links,
        // ));

        // classify.sort_by(|a, b| a.name.cmp(&b.name));
        // println!("{:?}", classify);
        // let mut alphbetical: Vec<Name_Id> = Vec::new();
        // let mut new = classify.clone();
        // for i in classify.iter() {
        //     if (alphbetical.len() == 0) {
        //         alphbetical.push(i);
        //     }
        // }
        // for i in classify.iter() {
        //     let mut lowestletter = i.name.clone();
        //     let mut index = 0;
        //     let mut lowindex = 0;
        //     for j in new.iter() {
        //         if (lowestletter > j.name) {
        //             println!(
        //                 "lowestnumber is {} and new name is {}",
        //                 lowestletter, j.name
        //             );
        //             lowestletter = j.name.clone();
        //             lowindex = index;
        //         }
        //         index = index + 1;
        //     }
        //     classify.remove(lowindex);
        //     new.remove(lowindex);
        // }
    }
    // println!("{:?}", classify);
    // Add remaining links in classify to the graph if they don't exist
    for item in classify.iter() {
        //println!("{:?}\n", item);
        let links = item.links.clone();
        //println!("{:?}", links);
        if (item.links.contains(&item.id)) {
            let links = remove(links, &item.id);

            graph.push(LinkNode::new(item.id, item.name.to_string(), links));
        } else {
            graph.push(LinkNode::new(item.id, item.name.to_string(), links));
        }
        //println!("graph      {:?}\n", graph)
    }
    //ensure_bidirectional_links(&mut graph);
    //getnodes(&mut graph);

    // println!("Total IDs: {}", id);
    // println!("Total Links: {}", num_links);
    //println!("{:?}", graph);

    graph
}

fn remove(links: Vec<usize>, id: &usize) -> Vec<usize> {
    let mut output = links.clone();
    let mut index = 0;
    let mut count = 0;
    for link in links {
        if &link == id {
            index = count;
            output.remove(index);
        }
        count += 1;
    }
    return output;
}

fn ensure_bidirectional_links(nodes: &mut Vec<LinkNode>) {
    // Iterate over all nodes
    for i in 0..nodes.len() {
        // For each link in the current node
        let node_id = nodes[i].id;
        let links = nodes[i].links.clone(); // Clone the links to avoid borrowing issues
        for &linked_id in links.iter() {
            // Find the linked node in the list of nodes
            if let Some(linked_node) = nodes.iter_mut().find(|n| n.id == linked_id) {
                // If the linked node doesn't already link back, add the reverse link
                if !linked_node.links.contains(&node_id) {
                    linked_node.links.push(node_id);
                }
            }
        }
    }
}

fn add_links(links: Vec<usize>, name_id: &mut Name_Id) {
    for link in links {
        name_id.links.push(link);
    }
}

fn checkforlinks(classify: &mut Vec<Name_Id>, id: &mut usize, doc: &str) -> Vec<usize> {
    let mut links: Vec<usize> = Vec::new();
    let node_id = *id; // The current node ID

    // Find all links in the document
    let link_names = find_links(doc);
    // println!("links");
    // println!("{:?}", link_names);

    for link in link_names {
        // Check if the link is already in classify
        let link_id = in_classify(&link, &classify);

        if let Some(link_id) = link_id {
            // Ensure no duplicate links
            if !links.contains(&link_id) {
                links.push(link_id);
            }
            // println!("{:?}", classify);
            // println!("the link id is {}", link_id);
            // println!("the node_id is  {}", node_id);
        } else {
            links.push(*id);
            *id += 1;
            // If link not found, add it
            //println!("New link found: {}", &link);
            classify.push(Name_Id::new(classify.len(), link.clone(), vec![]));
            // println!("psuhing this linknode in {:?}", classify);
            // *id += 1;
        }
    }

    links
}

fn getnodes(graph: &mut Graph) -> &mut Graph {
    let graph_len = graph.len();

    // Iterate through all nodes in the graph
    for node_index in 0..graph_len {
        let node_id = graph[node_index].id;

        // Create a temporary vector to hold the links that need to be added
        let mut new_links = Vec::new();

        // For each node, iterate through all other nodes
        for other_index in 0..graph_len {
            if node_index == other_index {
                continue; // Skip the same node
            }

            let other_node = &graph[other_index]; // Immutable borrow for the other node

            // If the other node contains a link to the current node, but the current node doesn't link back, mark it for addition
            if other_node.links.contains(&node_id)
                && !graph[node_index].links.contains(&other_node.id)
            {
                new_links.push(other_node.id);
            }
        }

        // Now that we're done iterating, we can safely modify the links of the current node
        graph[node_index].links.extend(new_links);
    }

    graph
}

fn getName_Id_by_id(id: usize, classify: &mut Vec<Name_Id>) -> &mut Name_Id {
    //println!("in getName_Id_by_id  the id trying to be found {}", id);
    for name in classify {
        if name.id == id {
            return name;
        }
    }
    todo!();
}

fn find_links(text: &str) -> Vec<String> {
    // Regex pattern to match most common types of URLs
    let url_pattern = r"(https?://|lb:)[^\s/$.?#].[^\s]*";
    let re = Regex::new(url_pattern).unwrap();

    // Collect all the matches into a Vec<String>
    let links: Vec<String> = re
        .find_iter(text)
        .map(|mat| {
            let url = mat.as_str().to_string();
            // Extract the website name from the URL
            extract_website_name(&url)
        })
        .collect();

    // Print each website name found
    for link in &links {
        //println!("Website name: {}", link);
    }

    // Return the website names
    links
}

fn extract_website_name(url: &str) -> String {
    // Remove "https://" or "http://" or "www." from the URL
    let domain = url
        .replace("https://", "")
        .replace("http://", "")
        .replace("www.", "");

    // Split by slashes and get the first part, which is the domain
    let parts: Vec<&str> = domain.split('/').collect();
    let domain_name = parts[0];

    // Get the base domain (youtube.com -> youtube) by splitting by dot and ignoring TLDs
    let name_parts: Vec<&str> = domain_name.split('.').collect();
    if name_parts.len() > 1 {
        name_parts[name_parts.len() - 2].to_string() // Extracts the main domain part
    } else {
        domain_name.to_string() // Fallback if no dots found
    }
}

fn in_classify(name: &String, classify: &Vec<Name_Id>) -> Option<usize> {
    // Search for the link in the classify vector and return its ID if found
    let mut id: Option<usize> = None;
    for linkinfo in classify {
        if &linkinfo.name == name {
            let optional_num: Option<usize> = Some(linkinfo.id);
            id = optional_num;
            break;
        }
    }
    id
}

fn getName_Id(name: &String, classify: &Vec<Name_Id>) -> Name_Id {
    for item in classify {
        if item.name == name.clone() {
            return item.clone();
        }
    }
    todo!();
    //return;
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
