mod pd;
mod rmq;

use std::env::{self};
use std::error::Error;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    dbg!(args.clone());

    real_main(args).await
}

async fn real_main(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    if args.len() != 3 {
        println!("Usage: {} <command> <file>", args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let file_path: &Path = Path::new(&args[2]);

    if !file_path.exists() {
        println!("File {} does not exist", file_path.display());
        std::process::exit(1);
    }

    let file_content = load_file(file_path)?;

    match command.as_ref() {
        "pd" => pd::pd(file_content),
        "rmq" => rmq::rmq(file_content),
        _ => {
            println!("Unknown command");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn load_file(file_path: &Path) -> Result<String, Box<dyn Error>> {
    let file_content = fs::read_to_string(file_path)?;
    Ok(file_content)
}
