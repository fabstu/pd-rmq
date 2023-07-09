mod heapsize;
mod indexed;
mod instances;
mod pd;
mod rmq;

extern crate graphannis_malloc_size_of as malloc_size_of;
#[macro_use]
extern crate graphannis_malloc_size_of_derive as malloc_size_of_derive;

//use graphannis_malloc_size_of::{MallocSizeOf, MallocSizeOfOps};

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

    match command.as_ref() {
        "pd" => pd::benchmark_and_check(file_path, None),
        "rmq" => rmq::rmq(file_path),
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
