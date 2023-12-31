use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use crate::debug::DEBUG;

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

    let mut line_buffer = String::new();

    for _i in 0..number_count {
        reader.read_line(&mut line_buffer)?;

        //println!("line: {}", line_buffer);

        // Trim to avoid newline.
        instance.numbers.push(line_buffer.trim().parse::<u64>()?);
        line_buffer.clear();
    }

    // Get the requests for the DS.
    for line in reader.lines() {
        let line = line?;

        if line.is_empty() {
            continue;
        }

        //println!("line: {}", line);

        // Trim to avoid newline.
        instance.queries.push(line.trim().parse::<u64>()?);
    }

    Ok(instance)
}

pub struct RMQInstance {
    pub numbers: Vec<u64>,
    pub queries: Vec<(usize, usize)>,
}

pub fn read_rmq_instance(path: &Path) -> Result<RMQInstance, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = io::BufReader::new(file);

    let mut instance = RMQInstance {
        numbers: Vec::new(),
        queries: Vec::new(),
    };

    let mut number_count_string = String::new();
    reader.read_line(&mut number_count_string)?;

    if DEBUG {
        println!("number_count_string: {}", number_count_string);
    }

    // Trim to avoid newline.
    let number_count = number_count_string.trim().parse::<i32>()?;

    let mut line_buffer = String::new();

    for _i in 0..number_count {
        reader.read_line(&mut line_buffer)?;

        //println!("line: {}", line_buffer);

        // Trim to avoid newline.
        instance.numbers.push(line_buffer.trim().parse::<u64>()?);
        line_buffer.clear();
    }

    // Get the requests for the DS.
    for line in reader.lines() {
        let line = line?;

        if line.is_empty() {
            continue;
        }

        //println!("line: {}", line);

        let vec: Vec<&str> = line.split(',').collect();

        assert_eq!(2, vec.len());

        // parse each string to u64
        let a: usize = vec[0].parse().unwrap();
        let b: usize = vec[1].parse().unwrap();

        // Trim to avoid newline.
        instance.queries.push((a, b));
    }

    Ok(instance)
}
