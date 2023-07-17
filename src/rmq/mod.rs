mod naive_fast;
mod naive_slow;
mod rmq_sparse;

use std::error::Error;
use std::fmt;

use crate::instances::RMQInstance;

use super::instances;

use std::path::Path;
use std::time::Instant;

use super::heapsize;
use super::report;

use crate::malloc_size_of::MallocSizeOf;
use crate::malloc_size_of::MallocSizeOfOps;

#[allow(unused_imports)]
use super::debug::DEBUG;

#[derive(Debug, PartialEq)]
pub enum RMQError {
    OutOfRange,
}

impl fmt::Display for RMQError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RMQError::OutOfRange => f.write_str("Out of range"),
        }
    }
}

impl Error for RMQError {}

pub fn rmq(path: &Path) {
    println!("rmq");

    //let path = Path::new("testdata/rmq_examples/rmq_example_1.txt");

    benchmark_and_check::<rmq_sparse::RMQSparse>(path, None);
}

pub trait RMQ {
    fn new(numbers: Vec<u64>) -> Self;
    fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError>;
}

pub fn benchmark_and_check<T: RMQ + MallocSizeOf>(path: &Path, want: Option<Vec<u64>>) {
    println!("rmq");

    let instance = instances::read_rmq_instance(path).unwrap();

    // Check correctness.
    if let Some(want) = want {
        let numbers = instance.numbers.clone();
        let rmq = T::new(numbers);

        for (i, query) in instance.queries.clone().iter().enumerate() {
            println!("Query nr {}: {:?}", i, query);
            let got = rmq
                .range_minimum_query(query.0 as usize, query.1 as usize)
                .unwrap();
            assert_eq!(want[i], got, "Query nr {}: {:?}", i, query);
        }
    }

    // Start benchmark
    benchmark::<T>(instance);
}

fn benchmark<T: RMQ + MallocSizeOf>(instance: RMQInstance) {
    // Clone numbers because we sort them.
    let numbers = instance.numbers;

    let start = Instant::now();

    let rmq = T::new(numbers);

    let queries_count = instance.queries.len();

    for (i, query) in instance.queries.iter().enumerate() {
        _ = rmq.range_minimum_query(query.0 as usize, query.1 as usize);

        if i % 100 == 0 {
            println!("Query nr {}/{}", i, queries_count);
        }
    }

    let duration = start.elapsed();

    let mut ops = MallocSizeOfOps::new(heapsize::platform::usable_size, None, None);
    let size = rmq.size_of(&mut ops);

    report::report("pd", duration, size);
}

// #[test]
// fn testing_rmq_test() {
//     let path = Path::new("testdata/rmq_examples/rmq_example_4.txt");
//
//     let want = vec![0, 0, 2, 2, 4, 4, 4, 7, 7, 7, 7];
//
//     benchmark_and_check(path, Some(want));
// }

#[test]
fn testing_rmq_naiveslow_benchmark1() {
    let path = Path::new("testdata/rmq_examples/rmq_example_1.txt");

    benchmark_and_check::<naive_slow::RMQNaiveSlow>(path, None);
}

#[test]
fn testing_rmq_sparse_benchmark1() {
    let path = Path::new("testdata/rmq_examples/rmq_example_1.txt");

    benchmark_and_check::<rmq_sparse::RMQSparse>(path, None);
}
