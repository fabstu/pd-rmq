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

use super::debug::DEBUG;

#[derive(Debug, PartialEq)]
pub enum RMQError {
    None,
}

impl fmt::Display for RMQError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RMQError::None => f.write_str("no error"),
        }
    }
}

impl Error for RMQError {}

#[derive(MallocSizeOf, Clone)]
pub struct RMQNaive {
    naive: Vec<Vec<u64>>,
}

impl RMQNaive {
    pub fn new(numbers: &mut Vec<u64>) -> Self {
        let n = numbers.len();

        // O(u^2). Could do u log n using consecutively less space for "to".
        //
        let mut naive = vec![vec![std::u64::MAX; n as usize]; n as usize];

        for i in 0..n {
            // Minimum to itself.
            naive[i][i] = numbers[i];

            for j in i + 1..n {
                // We only grow, so with each step we take on a new number.
                //
                // If the new number is smaller than the current, then record
                // it. Else keep the current minimum.
                naive[i as usize][j as usize] = std::cmp::min(naive[i][j - 1], numbers[j]);
            }
        }

        return Self { naive: naive };
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
        // Stay in range.
        assert!(from < self.naive.len());
        assert!(to < self.naive.len());

        return Ok(self.naive[from][to]);
    }
}

pub fn rmq(path: &Path) {
    println!("rmq");
}

pub fn benchmark_and_check(path: &Path, want: Option<Vec<u64>>) {
    println!("rmq");

    let instance = instances::read_rmq_instance(path).unwrap();

    // Check correctness.
    if let Some(want) = want {
        let mut numbers = instance.numbers.clone();
        let rmq = RMQNaive::new(&mut numbers);

        for (i, query) in instance.queries.clone().iter().enumerate() {
            println!("Query nr {}: {:?}", i, query);
            let got = rmq
                .range_minimum_query(query.0 as usize, query.1 as usize)
                .unwrap();
            assert_eq!(want[i], got, "Query nr {}: {:?}", i, query);
        }

        // assert_eq!(want, got);
    }

    // Start benchmark
    benchmark(instance);
}

fn benchmark(instance: RMQInstance) {
    // Clone numbers because we sort them.
    let mut numbers = instance.numbers.clone();

    let start = Instant::now();

    let rmq = RMQNaive::new(&mut numbers);

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
fn testing_rmq_benchmark1() {
    let path = Path::new("testdata/rmq_examples/rmq_example_1.txt");

    benchmark_and_check(path, None);
}
