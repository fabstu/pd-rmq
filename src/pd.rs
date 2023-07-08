use super::heapsize;
use super::indexed;
use super::instances;

use std::path::Path;
use std::time::Instant;

use crate::instances::PDInstance;
use crate::malloc_size_of::MallocSizeOf;
use crate::malloc_size_of::MallocSizeOfOps;

#[derive(MallocSizeOf)]
struct PD {
    // TODO: Implement structure for fast rank01 and select01.
    // Then: implement access and predecessor with them.
    upper: Vec<bool>,
    lower: Vec<bool>,
    upperBits: u64,
    lowerBits: u64,
}

impl PD {
    pub fn new(numbers: &mut Vec<u64>) -> Self {
        numbers.sort();

        let n = numbers.len();
        let nFloat = numbers.len() as f64;
        // n bit intgers
        let upperBits = nFloat.log2().ceil() as i32;
        // Is 44 for n=1.000.000 with upper=20!
        // Fixed ceil -> floor to not get 45.
        let lowerBits = (64.0 - nFloat.log2()).floor() as i32;

        let mut upperVec: Vec<bool> = vec![false; 2 ^ n];
        let mut lowerVec: Vec<bool> = Vec::with_capacity(numbers.len() * lowerBits as usize);

        for i in 0..numbers.len() {
            let number = numbers[i];

            // Remove lower bits for upper bits.
            let pi: usize = (number >> lowerBits) as usize;

            // Shift lowerBits times left to get 1 000 0000 000...
            // then -1 to make all lowerBits 1 while removing the leading one
            // that was too much.
            let lower = number & ((1 << lowerBits) - 1);

            upperVec[pi * i] = true;

            // Iterate lower bits using shifting the j-th bit to the first
            // position and then only keeping that one value around while
            // setting all other bits to zero.
            //
            // TODO: More efficient in single batch?
            for j in 0..lowerBits {
                let bit = (lower >> j) & 1;
                lowerVec.push(bit == 1);
            }
        }

        return Self {
            upper: upperVec,
            lower: lowerVec,
            upperBits: upperBits as u64,
            lowerBits: lowerBits as u64,
        };
    }

    // Lower bit access:
    // i - 1 bits davor * Anzah-bits die die zahlen lang sind
    // -> einzelne bits lesen.
    pub fn access(&self, i: u64) -> bool {
        let upper_part = self.upper.select0(i) - i;

        let mut lower_part = 0;

        for j in 0..self.lowerBits as u64 {
            let bit = self.lower[(i * self.lowerBits + j) as usize];

            lower_part = lower_part | (if bit { 1 } else { 0 } << j);
        }

        return upper_part << self.lowerBits | lower_part;
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
    let mut numbers = instance.numbers.clone();

    let start = Instant::now();

    let pd = PD::new(&mut numbers);

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
        let mut numbers = instance.numbers.clone();
        let pd = PD::new(&mut numbers);
        let mut got = Vec::<usize>::new();

        for query in instance.queries.clone() {
            got.push(pd.pred(query));
        }

        assert_eq!(want, got);
    }

    // Start benchmark
    benchmark(instance);
}

#[test]
fn greeting_contains_name() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_4.txt");

    let want = vec![4, 4, 3, 3];

    benchmark_and_check(path, Some(want));
}
