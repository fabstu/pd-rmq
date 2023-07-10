use std::time::Duration;

pub fn report(algo: &str, time: Duration, space: usize) {
    println!(
        "RESULT algo={} nameFabian_Sturm time={} space={}",
        algo,
        time.as_millis(),
        space
    );
}
