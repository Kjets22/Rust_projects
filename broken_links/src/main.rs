use clap::error::Result;
use clap::Parser;
use linkify::{LinkFinder, Links};
use std::{
    fs::{self, read_to_string},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(version, about = "Checks for broken links in a directory.", long_about = None)]
struct Args {
    /// Give a folder with broken links path
    #[arg(short, long)]
    broken_link_path: Option<PathBuf>,
}

//takes a direcotry and opens it converts to content in the files and runs a method
fn directory_to_file_action<F>(files: &path, method: F) -> Result<()>
where
    F: Fn(String),
{
    let files = fs::read_dir(files)?;
    files
        .map(|file| method(read_to_string(file.unwrap().path()).unwrap()))
        .collect::<Vec<_>>();
    Ok(())
}

//find the links through the linkify crate
fn find_links(content: String) {
    println!("in find links");
    let finder = LinkFinder::new();
    let mut links = finder.links(&content);
    let trying = links.nth(0);
    print_brokenlinks(links)
}

//takes the links and sees if they actualy go anywhere
fn print_brokenlinks(links: Links) {
    println!("in print_brokenlinks");
    for link in links {
        let link = link.as_str();
        match reqwest::blocking::get(link) {
            Ok(_l) => {
                continue;
            }
            Err(_e) => {
                println!("{}\n", link);
            }
        }
    }
}

//main method
fn main() {
    let cli = Args::parse();
    if let Some(path) = cli.broken_link_path.as_deref() {
        directory_to_file_action(path, find_links);
    }
}
