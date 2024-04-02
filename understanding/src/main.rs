use clap::{command, Arg, ArgMatches};
use linkify::LinkFinder;
use reqwest;
use std::fs;

fn cli() -> ArgMatches {
    command!()
        .arg(
            Arg::new("broken_links")
                .short('b')
                .long("brokenlinks")
                .help("this is broken links "),
        )
        .arg(
            Arg::new("continue")
                .short('c')
                .long("con")
                .help("this is to continue and avoid the link"),
        )
        .get_matches()
}

fn broken_links(folder_directory: &String) {
    let folder_directory = folder_directory;
    println!("{}", folder_directory);
    match fs::read_dir(folder_directory) {
        Ok(open_directory) => {
            for file in open_directory {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        let finder = LinkFinder::new();
                        let links: String<_> = finder.links(content).collect();
                        for link in links {}
                    }
                    Err(_e) => {}
                }
            }
        }
        Err(e) => {}
    }
}

fn main() {
    let matches = cli();
    if matches.contains_id("broken_links") {
        broken_links(matches.get_one::<String>("broken_links").unwrap());
    }
}
