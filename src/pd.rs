use super::indexed;
use std::time::Instant;

use super::heapsize;
use crate::malloc_size_of::MallocSizeOf;
use crate::malloc_size_of::MallocSizeOfOps;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

struct PDInstance {
    numbers: Vec<u64>,
    queries: Vec<u64>,
}

fn read_pd_instance(path: &Path) -> Result<PDInstance, Box<dyn Error>> {
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

pub fn pd_simple(path: &Path) {
    println!("pd");

    let instance = read_pd_instance(path).unwrap();

    let start = Instant::now();

    let bit_vec = indexed::IndexedBitVec {
        data: vec![true, false, true, false],
    };

    let duration = start.elapsed();

    let mut ops = MallocSizeOfOps::new(heapsize::platform::usable_size, None, None);
    let size = bit_vec.size_of(&mut ops);

    indexed::report("pd".to_string(), duration, size);
}
