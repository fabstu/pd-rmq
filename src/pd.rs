use super::heapsize;
use super::indexed;
use super::instances;

use std::path::Path;
use std::time;
use std::time::Instant;

use crate::instances::PDInstance;
use crate::malloc_size_of::MallocSizeOf;
use crate::malloc_size_of::MallocSizeOfOps;

#[derive(MallocSizeOf)]
struct PD {
    data: Vec<bool>,
}

impl PD {
    pub fn new(numbers: Vec<u64>) -> Self {
        Self { data: Vec::new() }
    }

    pub fn pred(&self, i: u64) -> usize {
        return 3;
    }
}

// struct Report {
//     algo: String,
//     time: time::Duration,
//     space: usize,
// }

fn benchmark(instance: PDInstance) {
    let start = Instant::now();

    let pd = PD::new(instance.numbers);

    for query in instance.queries {
        _ = pd.pred(query);
    }

    let duration = start.elapsed();

    let mut ops = MallocSizeOfOps::new(heapsize::platform::usable_size, None, None);
    let size = pd.size_of(&mut ops);

    indexed::report("pd".to_string(), duration, size);
}

pub fn benchmark_and_check(path: &Path, want: Option<Vec<usize>>) {
    println!("pd");

    let instance = instances::read_pd_instance(path).unwrap();

    // Check correctness.
    if let Some(want) = want {
        let pd = PD::new(instance.numbers.clone());
        let mut got = Vec::<usize>::new();

        for query in instance.queries.clone() {
            got.push(pd.pred(query));
        }

        assert_eq!(want, got);
    }

    // Start benchmark
    benchmark(instance);
}
