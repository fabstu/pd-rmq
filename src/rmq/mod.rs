mod naive_fast;
mod naive_slow;
mod rmq_spanning_blocks;
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

pub fn rmq(path: &Path, out: Option<String>) {
    if DEBUG {
        println!("rmq");
    }

    //let path = Path::new("testdata/rmq_examples/rmq_example_1.txt");

    benchmark_and_check_path::<rmq_spanning_blocks::RMQSpanningBlocks>(path, None, out);
    // benchmark_and_check_path::<rmq_sparse::RMQSparse>(path, None);
}

pub trait RMQ {
    fn new(numbers: Vec<u64>) -> Self;
    fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError>;
}

pub fn benchmark_and_check_path<T: RMQ + MallocSizeOf>(
    path: &Path,
    want: Option<Vec<usize>>,
    out: Option<String>,
) {
    let instance = instances::read_rmq_instance(path).unwrap();
    benchmark_and_check_instance::<T>(instance, want, out);
}

pub fn benchmark_and_check_instance<T: RMQ + MallocSizeOf>(
    instance: RMQInstance,
    want: Option<Vec<usize>>,
    out: Option<String>,
) {
    if DEBUG {
        println!("rmq");
    }

    // Check correctness.
    if let Some(want) = want {
        let numbers = instance.numbers.clone();
        let rmq = T::new(numbers.clone());

        for (i, query) in instance.queries.clone().iter().enumerate() {
            if DEBUG {
                println!("Query nr {}: {:?}", i, query);
            }
            let got = rmq
                .range_minimum_query(query.0 as usize, query.1 as usize)
                .unwrap();

            // Only check as long as we have enough want-elements.
            if i < want.len() {
                assert!(want[i] < numbers.len());
                assert!(want[i] <= query.1, "Query: {:?}, want: {}", query, want[i]);
                assert!(want[i] >= query.0, "Query: {:?}, want: {}", query, want[i]);
                assert!(
                    got < numbers.len(),
                    "Query: {:?} got: {} numbers.len: {}",
                    query,
                    got,
                    numbers.len()
                );
                assert!(got <= query.1, "Query: {:?}, got: {}", query, got);
                assert!(got >= query.0, "Query: {:?}, got: {}", query, got);

                assert_eq!(
                    want[i], got,
                    "Query check nr {}: {:?} want-resolved: {} got-resolved: {}",
                    i, query, numbers[want[i] as usize], numbers[got as usize]
                );
            }

            assert!(got < numbers.len());
        }
    }

    // Start benchmark
    benchmark::<T>(instance, out);
}

#[allow(dead_code)]
pub fn benchmark_and_check_with_checker<T: RMQ + MallocSizeOf, Checker: RMQ + MallocSizeOf>(
    path: &Path,
    want_number_checked: isize,
) {
    if DEBUG {
        println!("rmq");
    }

    let instance = instances::read_rmq_instance(path).unwrap();

    // Create want-vector to check against from checker.
    let checker: Checker = Checker::new(instance.numbers.clone());
    let mut want: Vec<usize> = Vec::new();

    for (i, query) in instance.queries.clone().iter().enumerate() {
        // Only check up to want_number_checked.
        if want_number_checked > 0 && i as isize >= want_number_checked {
            break;
        }

        let got = checker
            .range_minimum_query(query.0 as usize, query.1 as usize)
            .unwrap();

        if DEBUG {
            println!("Query nr {}: {:?} -> {}", i, query, got);
        }

        want.push(got);
    }
}

#[allow(dead_code)]
pub fn benchmark_and_check_with_checker_parallel<
    T: RMQ + MallocSizeOf,
    Checker: RMQ + MallocSizeOf,
>(
    path: &Path,
    want_number_checked: isize,
    out: Option<String>,
) {
    println!("rmq");

    let instance = instances::read_rmq_instance(path).unwrap();

    let numbers = instance.numbers.clone();

    // Create want-vector to check against from checker.
    let checker: Checker = Checker::new(instance.numbers.clone());

    let under_test: T = T::new(instance.numbers.clone());

    for (i, query) in instance.queries.clone().iter().enumerate() {
        // Only check up to want_number_checked.
        if want_number_checked > 0 && i as isize >= want_number_checked {
            break;
        }

        let want = checker
            .range_minimum_query(query.0 as usize, query.1 as usize)
            .unwrap();

        if DEBUG {
            println!("Query nr {}: {:?}", i, query);
        }
        let got = under_test
            .range_minimum_query(query.0 as usize, query.1 as usize)
            .unwrap();

        // Only check as long as we have enough want-elements.
        if want_number_checked > 0 && i < want_number_checked as usize {
            assert!(want < numbers.len());
            assert!(want <= query.1, "Query: {:?}, want: {}", query, want);
            assert!(want >= query.0, "Query: {:?}, want: {}", query, want);
            assert!(
                got < numbers.len(),
                "Query: {:?} got: {} numbers.len: {}",
                query,
                got,
                numbers.len()
            );
            assert!(got <= query.1, "Query: {:?}, got: {}", query, got);
            assert!(got >= query.0, "Query: {:?}, got: {}", query, got);

            assert_eq!(
                want, got,
                "Query check nr {}: {:?} want-resolved: {} got-resolved: {}",
                i, query, numbers[want], numbers[got]
            );
        }

        assert!(got < numbers.len());

        if DEBUG {
            println!("Query nr {}: {:?} -> {}", i, query, want);
        }
    }

    benchmark::<T>(instance, out);
}

fn benchmark<T: RMQ + MallocSizeOf>(instance: RMQInstance, out: Option<String>) {
    // Clone numbers because we sort them.
    let numbers = instance.numbers;

    let mut got_all: Vec<u64> = Vec::with_capacity(numbers.len());

    let start = Instant::now();

    let rmq = T::new(numbers);

    let queries_count = instance.queries.len();

    for (i, query) in instance.queries.iter().enumerate() {
        got_all.push(
            rmq.range_minimum_query(query.0 as usize, query.1 as usize)
                .unwrap() as u64,
        );

        if DEBUG {
            if i % 100 == 0 {
                println!("Query nr {}/{}", i, queries_count);
            }
        }
    }

    let duration = start.elapsed();

    let mut ops = MallocSizeOfOps::new(heapsize::platform::usable_size, None, None);
    let size = rmq.size_of(&mut ops);

    report::write_out(out, got_all);

    report::report("rmq", duration, size);
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

    benchmark_and_check_path::<naive_slow::RMQNaiveSlow>(path, None, None);
}

#[test]
fn testing_rmq_sparse_benchmark1() {
    let path = Path::new("testdata/rmq_examples/rmq_example_1.txt");

    // Only check first 2k because naive_slow is super slow.
    benchmark_and_check_with_checker::<rmq_sparse::RMQSparse, naive_slow::RMQNaiveSlow>(path, 2000);
}

#[test]
fn testing_rmq_spanning_benchmark1() {
    let path = Path::new("testdata/rmq_examples/rmq_example_1.txt");

    benchmark_and_check_with_checker_parallel::<
        rmq_spanning_blocks::RMQSpanningBlocks,
        rmq_sparse::RMQSparse,
    >(path, -1, None);
}

#[test]
fn testing_rmq_spanning_benchmark2() {
    let path = Path::new("testdata/rmq_examples/rmq_example_2.txt");

    benchmark_and_check_with_checker_parallel::<
        rmq_spanning_blocks::RMQSpanningBlocks,
        rmq_sparse::RMQSparse,
    >(path, -1, None);
}
