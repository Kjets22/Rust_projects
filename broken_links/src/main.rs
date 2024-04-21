use clap::error::Result;
use clap::Parser;
use lazy_static::lazy_static;
use linkify::{LinkFinder, Links};
use std::{
    fs::{self, read_to_string},
    path::{Path, PathBuf},
};
lazy_static! {
    static ref FINDER: LinkFinder = LinkFinder::new();
}

#[derive(Parser, Debug)]
#[command(version, about = "Checks for broken links in a directory.", long_about = None)]
/// CLI
struct Args {
    /// Give a folder with broken links path
    #[arg(short, long)]
    broken_link_path: Option<PathBuf>,
}

/// takes a direcotry and opens it converts to content in the files and runs a method
fn directory_to_file_action<F>(files: &Path, method: &F) -> Result<()>
where
    F: Fn(String),
{
    let files = fs::read_dir(files)?;
    for file in files {
        let file = file.unwrap().path();
        match read_to_string(&file) {
            Ok(content) => {
                method(content);
            }
            Err(_) => {
                let _ = directory_to_file_action(&file, method);
            }
        }
    }
    Ok(())
}

/// find the links through the linkify crate
fn find_links(content: String) {
    let links = FINDER.links(&content);
    print_brokenlinks(links)
}

/// takes the links and sees if they actualy go anywhere
fn print_brokenlinks(links: Links) {
    for link in links {
        let link = link.as_str();
        match reqwest::blocking::get(link) {
            Ok(result) => {
                if result.status().is_success() {
                    // if finds url and not 404 anything in the 200s
                } else {
                    // if its a 404 or anything else
                    println!("{}\n", link);
                }
            }
            Err(_e) => {
                println!("{}\n", link);
            }
        }
    }
}

/// main method
fn main() {
    let cli = Args::parse();
    if let Some(path) = cli.broken_link_path.as_deref() {
        let _ = directory_to_file_action(path, &find_links);
    }
}
