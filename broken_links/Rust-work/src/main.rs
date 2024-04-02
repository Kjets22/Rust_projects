use clap::{command,Arg,ArgMatches};
use url::Url;

fn cli()->ArgMatches{
	let match_result=command!()
	.about("give a folder with url files, the program will return broken urls")
	.arg(
		Arg::new("folder")
		.short('f')
		.help("input a folder directory to get broken urls")
//		.value_parser(value_parser!(PathBuf))
	)
	.get_matches();
	match_result
}

fn main() {
	let match_result=cli();
	if let Some(folder_directory) = match_result.get_one::<String>("folder") {
//		let folder_directory=match_result.get_one::<String>("folder");
		match std::fs::read_dir(folder_directory){
			Ok(folder_path)=>{
				for file_path in folder_path {
					match file_path{
						Ok(
//					println!("succesfull");
//					let file_path=file_path.to_string(); 	
					match Url::parse(&file_path){
						Ok(url)=>{
							continue;
						}
						Err(e)=>{
							println!("{}",file_path);
						}	
				//chekc if the file path is a working url probaly another mathc statement where if it is not a file run continue
					}
				}
			}
			Err(_e)=>{
				println!("This folder directory is not valid");
			}
		}
	}
}
