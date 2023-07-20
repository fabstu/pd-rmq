use std::time::Duration;

pub fn report(algo: &str, time: Duration, space: usize) {
    println!(
        "RESULT algo={} name=Fabian_Sturm time={} space={}",
        algo,
        time.as_millis(),
        space
    );
}
