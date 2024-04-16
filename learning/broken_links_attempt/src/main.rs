use clap::Parser; use std::collections::LinkedList; use std::fs::{self, read_to_string, ReadDir}; use std::{io, path}; use std::path::{Path, PathBuf}; use linkify::LinkFinder;
#[derive(Parser, Debug)] 
#[command(version, about = "Checks for broken links in a directory.", long_about = None)]
struct Args {
    /// Give a folder with broken links path 
    #[arg(short, long)] 
    broken_link_path: Option<PathBuf>,
}
//so what this does it takes a derectory and then opens its into each of its files 
//then it gets the contents of each file and sends it to a method
fn directory_to_file_action<F>(files: &Path, method: F )  {//here what i am doing taking a method as a parameter
    where
        F: FnMut(Path){
        let files = fs::read_dir(files)?;            
        for file in files {
            let file= file?;//this make it so it is dirEntry and no longer a result enum                  
            let file= file.path();
            let content=read_to_string(file)?;
            
        }
    }
                                         
}

fn find_links(content:string){
    let finder=LinkFinder::new();
    let links=finder.links(content);
       
}

fn main()  {                  
    let cli = Args::parse();
    if let Some(path) = cli.broken_link_path.as_deref() {
        directory_to_file_action(path,|path| find_broken_links(path));                      
    }
}    //let args = Args::parse();
    //if let Some(broken_link_path) = args.broken_link_path.as_deref() {
    //     let files = fs::read_dir(broken_link_path)?;
    //     for file in files {
    //         broken_links(file.path());
    //     }
    // }
    //match args.broken_link_path{
    //    println!("exist");
    //}
    //println!(
    //    "print brokenlink {}",
    //    args.broken_link_path.unwrap().display()
    //);

