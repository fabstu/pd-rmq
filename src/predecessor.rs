use super::bitvector;
use super::heapsize;
use super::instances;
use super::report;

use std::path::Path;
use std::time::Instant;

use crate::bitvector::MyError;
use crate::instances::PDInstance;
use crate::malloc_size_of::MallocSizeOf;
use crate::malloc_size_of::MallocSizeOfOps;

#[derive(MallocSizeOf)]
struct PD {
    numbers_count: u64,
    upper: bitvector::Bitvector,
    lower: Vec<bool>,
    upper_bits: u64,
    lower_bits: u64,
}

impl PD {
    fn split(&self, i: u64) -> (u64, usize) {
        return Self::split_with_bit_distribution(i, self.lower_bits, self.upper_bits);
    }

    #[allow(unused_variables)]
    fn split_with_bit_distribution(i: u64, lower_bits: u64, upper_bits: u64) -> (u64, usize) {
        // Remove lower bits for upper bits.
        let upper: usize = (i >> lower_bits) as usize;

        // Shift lowerBits times left to get 1 000 0000 000...
        // then -1 to make all lowerBits 1 while removing the leading one
        // that was too much.
        let lower = i & ((1 << lower_bits) - 1);

        return (lower, upper);
    }

    pub fn new(numbers: &mut Vec<u64>) -> Self {
        numbers.sort();

        // Biggest number in universe.
        let u = numbers[numbers.len() - 1];

        // Number of numbers.
        let n = numbers.len();
        let n_float = numbers.len() as f64;

        // n bit intgers
        let upper_bits = n_float.log2().ceil() as i32;
        // Is 44 for n=1.000.000 with upper=20!
        // Fixed ceil -> floor to not get 45.
        let lower_bits = ((u as f64).log2() - n_float.log2()).ceil() as i32;

        let mut upper_vec: Vec<bool> = vec![false; 2 ^ n];
        let mut lower_vec: Vec<bool> = Vec::with_capacity(numbers.len() * lower_bits as usize);

        let pi_divisor = 2u32.pow(upper_bits as u32) as u64;

        // Sort numbers.

        for i in 0..numbers.len() {
            let number = numbers[i];

            let (lower, _) =
                Self::split_with_bit_distribution(number, lower_bits as u64, upper_bits as u64);

            // Calculat pi.
            //
            // Basically gets used
            let pi = (number / pi_divisor) as usize;

            // let pi: usize = (number >> lowerBits) as usize;

            // // Shift lowerBits times left to get 1 000 0000 000...
            // // then -1 to make all lowerBits 1 while removing the leading one
            // // that was too much.
            // let lower = number & ((1 << lowerBits) - 1);

            upper_vec[pi * i] = true;

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
            numbers_count: n as u64,
            upper: bitvector::Bitvector::new(upper_vec),
            lower: lower_vec,
            upper_bits: upper_bits as u64,
            lower_bits: lower_bits as u64,
        };
    }

    // Lower bit access:
    // i - 1 bits davor * Anzah-bits die die zahlen lang sind
    // -> einzelne bits lesen.
    #[allow(dead_code)]
    pub fn access(&self, i: u64) -> Result<u64, MyError> {
        let upper_part = self.upper.select1(i.try_into().unwrap())? - i;

        let mut lower_part = 0;

        for j in 0..self.lower_bits as u64 {
            let bit = self.lower[(i * self.lower_bits + j) as usize];

            lower_part = lower_part | (if bit { 1 } else { 0 } << j);
        }

        return Ok((upper_part << self.lower_bits | lower_part) as u64);
    }

    fn bits_to_u64(bits: &[bool]) -> u64 {
        let mut result = 0;

        for i in 0..bits.len() {
            if bits[i] {
                result = result | (1 << i);
            }
        }

        return result;
    }

    fn decrement_min_zero(v: u64) -> u64 {
        if v == 0 {
            return 0;
        }

        return v - 1;
    }

    pub fn pred(&self, i: u64) -> Result<u64, MyError> {
        // Split into lower and upper.
        //
        // Quqestion: MSB is supposed to include zeroes and ones both, right?
        // Otherwise, lower becomes dynamic.
        let (lower, msb) = self.split(i);

        // Index in upper vector of the start of the bucket.
        let p = self.upper.select0(msb as u64)?;

        // Indexes up to and including the first in the bucket
        //
        // Because the prefix sum up to and including that one
        // is the index in the lower vector, we can start scanning using this.
        let ith_in_original_numbers = self.upper.rank1(p + 1);

        // p in upper is already false.
        if self.upper.get(p + 1) == false {
            // We are in a higher bucket, so the bucket was empty, so we need to
            // take the last from a smaller bucket and return that.
            return self.access(self.upper.rank1(p));
        }

        // This bucket is non-empty.
        //
        // ith points to the first in that bucket.
        //
        // Two cases can happen:
        // a) I find bigger in this bucket.
        // a1) ith is bigger: return self.access(ith-1)
        // a1.1) if ith-1 is zero, return self.access(0)
        // a1.2) if ith-1 is non-zero, return self.access(ith-1)
        // a2) other-than-ith is bigger: return self.access(bigger - 1)
        // b) I find none bigger in this bucket, which means lower would be
        //    biggest because it is in that bucket. Return the last in this
        //    bucket.

        // Iterate in lower_vec by lower_bits.
        //
        // Can be sped up getting bucket boundaries and
        // halving each time for O(n log n).
        for i in ith_in_original_numbers..self.numbers_count {
            let start = (i * self.lower_bits) as usize;
            let end = start + self.lower_bits as usize;

            let bits: &[bool] = &self.lower[start..end];
            if Self::bits_to_u64(bits) > lower {
                // a)
                return self.access(Self::decrement_min_zero(i));
            }
        }

        // b)
        return self.access(Self::decrement_min_zero(ith_in_original_numbers));
    }
}

// struct Report {
//     algo: String,
//     time: time::Duration,
//     space: usize,
// }

fn benchmark(instance: PDInstance) {
    // Clone numbers because we sort them.
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
            got.push(pd.pred(query).unwrap());
        }

        assert_eq!(want, got);
    }

    // Start benchmark
    benchmark(instance);
}

#[test]
fn testing_pd_benchmark() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_4.txt");

    let want = vec![];

    benchmark_and_check(path, Some(want));
}
