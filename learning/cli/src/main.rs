use clap::Parser;
use std::path::PathBuf;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Give a folder with broken links path
    #[arg(short, long, required = true)]
    broken_link_path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    println!(
        "print brokenlink {}",
        args.broken_link_path.unwrap().display()
    );
}
