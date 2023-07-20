use std::io::Write;
use std::{fs::OpenOptions, time::Duration};

pub fn report(algo: &str, time: Duration, space: usize) {
    println!(
        "RESULT algo={} name=Fabian_Sturm time={} space={}",
        algo,
        time.as_millis(),
        space
    );
}
pub fn write_out(out: Option<String>, got_all: Vec<u64>) {
    match out {
        Some(out) => {
            //
            // Write output.
            //
            println!("Writing output: out={}", out);

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(false)
                .open(&out);
            if file.is_err() {
                println!(
                    "Could not open output file: {} error: {}",
                    out,
                    file.unwrap_err()
                );
                std::process::exit(1);
            }

            let out_string = got_all
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            file.unwrap().write_all(out_string.as_bytes()).unwrap();
        }
        None => {}
    }
}
