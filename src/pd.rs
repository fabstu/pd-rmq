use super::heapsize;
use super::indexed;
use super::instances;
use super::report;

use std::path::Path;
use std::time::Instant;

use crate::instances::PDInstance;
use crate::malloc_size_of::MallocSizeOf;
use crate::malloc_size_of::MallocSizeOfOps;

#[derive(MallocSizeOf)]
struct PD {
    // TODO: Implement structure for fast rank01 and select01.
    // Then: implement access and predecessor with them.
    upper: indexed::Bitvector,
    lower: Vec<bool>,
    upper_bits: u64,
    lower_bits: u64,
}

impl PD {
    fn split(&self, i: u64) -> (u64, usize) {
        return Self::split_with_bit_distribution(i, self.lower_bits, self.upper_bits);
    }

    fn split_with_bit_distribution(i: u64, lower_bits: u64, upper_bits: u64) -> (u64, usize) {
        // Remove lower bits for upper bits.
        let pi: usize = (i >> lower_bits) as usize;

        // Shift lowerBits times left to get 1 000 0000 000...
        // then -1 to make all lowerBits 1 while removing the leading one
        // that was too much.
        let lower = i & ((1 << lower_bits) - 1);

        return (lower, pi);
    }

    pub fn new(numbers: &mut Vec<u64>) -> Self {
        numbers.sort();

        let n = numbers.len();
        let n_float = numbers.len() as f64;
        // n bit intgers
        let upper_bits = n_float.log2().ceil() as i32;
        // Is 44 for n=1.000.000 with upper=20!
        // Fixed ceil -> floor to not get 45.
        let lower_bits = (64.0 - n_float.log2()).floor() as i32;

        let mut upper_vec: Vec<bool> = vec![false; 2 ^ n];
        let mut lower_vec: Vec<bool> = Vec::with_capacity(numbers.len() * lower_bits as usize);

        for i in 0..numbers.len() {
            let number = numbers[i];

            let (lower, pi) =
                Self::split_with_bit_distribution(number, lower_bits as u64, upper_bits as u64);

            // let pi: usize = (number >> lowerBits) as usize;

            // // Shift lowerBits times left to get 1 000 0000 000...
            // // then -1 to make all lowerBits 1 while removing the leading one
            // // that was too much.
            // let lower = number & ((1 << lowerBits) - 1);

            upper_vec[pi as usize * i] = true;

            // Iterate lower bits using shifting the j-th bit to the first
            // position and then only keeping that one value around while
            // setting all other bits to zero.
            //
            // TODO: More efficient in single batch?
            for j in 0..lower_bits {
                let bit = (lower >> j) & 1;
                lower_vec.push(bit == 1);
            }
        }

        return Self {
            upper: indexed::Bitvector::new(upper_vec),
            lower: lower_vec,
            upper_bits: upper_bits as u64,
            lower_bits: lower_bits as u64,
        };
    }

    // Lower bit access:
    // i - 1 bits davor * Anzah-bits die die zahlen lang sind
    // -> einzelne bits lesen.
    pub fn access(&self, i: u64) -> u64 {
        let upper_part = self.upper.select1(i.try_into().unwrap()) - i;

        let mut lower_part = 0;

        for j in 0..self.lower_bits as u64 {
            let bit = self.lower[(i * self.lower_bits + j) as usize];

            lower_part = lower_part | (if bit { 1 } else { 0 } << j);
        }

        return (upper_part << self.lower_bits | lower_part) as u64;
    }

    pub fn pred(&self, i: u64) -> u64 {
        // Split into lower and upper.
        let (lower, pi) = self.split(i);

        let mut lower_bound = self.upper.select0(pi as u64);
        let upper_bound_excluding = self.upper.select0(pi as u64 + 1);

        if lower_bound == upper_bound_excluding - 1 {
            // Bucket is empty. Just get the next-lower value.
            return self.upper.select1(self.upper.rank1(i - 1));
        }

        // lower_bound and upper_bound_excluding are zeroes in self.upper
        //
        // (except if lower_bound is 0, then it is a 1) in self.upper)
        if lower_bound != 0 {
            // Move lower_bound to 1st 1 in bucket.
            lower_bound += 1;
        }

        // Question is: Do I always find a value smaller than lower-bits of i?
        // What if I don't?
        //
        // And.. Do I even have to scan multiple entries?
        // Yes, until I find a bigger value than lower-bits of i.
        // But.. isn't that the other way around compared to what was expected.
        //
        // So..
        // a) Walk down from 1 in bucket until smaller is found in lower.
        //    If smaller is not found and we go past lower_bound, choose
        //    1 before this bucket.
        //
        // b) Walk up from lower_bound and find first that is bigger in
        //    self.lower. If one is found, return the pos of the 1 before.
        //    If none is bigger, then lower is biggest.
        //
        //
        loop {}

        // Bucket is not empty.
        //
        // Find first smaller than lower in lower.
        for j in lower_bound..upper_bound_excluding {}

        self.upper.select0(i) + i;

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

    report::report("pd", duration, size);
}

pub fn benchmark_and_check(path: &Path, want: Option<Vec<u64>>) {
    println!("pd");

    let instance = instances::read_pd_instance(path).unwrap();

    // Check correctness.
    if let Some(want) = want {
        let mut numbers = instance.numbers.clone();
        let pd = PD::new(&mut numbers);
        let mut got = Vec::<u64>::new();

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
