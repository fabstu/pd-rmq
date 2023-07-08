use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

pub struct PDInstance {
    pub numbers: Vec<u64>,
    pub queries: Vec<u64>,
}

pub fn read_pd_instance(path: &Path) -> Result<PDInstance, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = io::BufReader::new(file);

    let mut instance = PDInstance {
        numbers: Vec::new(),
        queries: Vec::new(),
    };

    let mut number_count_string = String::new();
    reader.read_line(&mut number_count_string)?;

    println!("number_count_string: {}", number_count_string);

    // Trim to avoid newline.
    let number_count = number_count_string.trim().parse::<i32>()?;

    let mut lineBuffer = String::new();

    for _i in 0..number_count - 1 {
        reader.read_line(&mut lineBuffer)?;

        println!("line: {}", lineBuffer);

        // Trim to avoid newline.
        instance.numbers.push(lineBuffer.trim().parse::<u64>()?);
        lineBuffer.clear();
    }

    // Get the requests for the DS.
    for line in reader.lines() {
        let line = line?;

        if line.is_empty() {
            continue;
        }

        println!("line: {}", line);

        // Trim to avoid newline.
        instance.queries.push(line.trim().parse::<u64>()?);
    }

    Ok(instance)
}
