use super::bitvector;
use super::heapsize;
use super::instances;
use super::report;

use std::cmp::max;
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

        // TODO:
        // Biggest number in universe.
        //
        // Except.. this means that for u=10, lower_bits is zero.
        // This means that the lower_bits cannot be stored despite needing
        // storing, which means self.access(i) is broken.
        //
        // But.. I can't selectively use a higher lower_bits,
        // because lower_bits is used to reconstruct the lower bits.
        //
        // Idea: Can just choose appropriate lower_bits,
        // since upper_Bits is never used.
        //
        // Its only used for pi_divisor and for initial splitting into lower.
        // But since it was zero, ...?
        //
        let u = numbers[numbers.len() - 1];

        // Number of numbers.
        let n = numbers.len();
        let n_float = numbers.len() as f64;

        // n bit intgers

        // Switched lower_bits and upper_bits.
        // Except.. access expects to find true in upper for access.
        //
        // And.. also for predecessor. It specifically uses true
        // to determine at what indices to start searching in lower
        // by counting and select0(msb)-ing.
        //
        // So... choosing both the same might work.

        let lower_bits = n_float.log2().ceil() as i32;
        // Is 44 for n=1.000.000 with upper=20!
        // Fixed ceil -> floor to not get 45.
        // let upper_bits = max(((u as f64).log2() - n_float.log2()).ceil() as i32, 2);
        let upper_bits = n_float.log2().ceil() as i32;

        let mut upper_vec: Vec<bool> = vec![false; 2 * n + 1];
        let mut lower_vec: Vec<bool> = Vec::with_capacity(numbers.len() * lower_bits as usize);

        // How do I handle upper_bits = 0?
        // a) Use 1 by default.
        // b) Sepcial-case insertion into upper (skipping it).
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

            println!(
                "Setting to true: number: {} i: {} pi: {} pi+i: {} pi_divisor: {}",
                number,
                i,
                pi,
                pi + i,
                pi_divisor
            );

            // Crash here due to pi + i going over upper_vec.
            // 38 mio vs 2 mio.
            //
            // Setting to true: number: 40539300726434 i: 0 pi: 38661289 pi+i: 38661289 pi_divisor: 1048576
            // thread 'predecessor::testing_pd_benchmark1' panicked at 'index out of bounds: the len is 2000001 but the index is 38661289', src/predecessor.rs:120:13
            upper_vec[pi + i] = true;

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

        println!(
            "PD::new - numbers: {:?} upper: {:?} lower: {:?} upper_bits: {} lower_bits: {}",
            numbers, upper_vec, lower_vec, upper_bits, lower_bits
        );

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
        println!(
            "access({}) - upper_select1({}): {}",
            i,
            i,
            self.upper.select1(i)?
        );

        assert!(i < self.numbers_count, "i must be smaller than n");

        // Crashes for i == 1 because upper.select1(1) returns 0.
        // Isn't i supposed to be something akin to be added to?
        //
        // Problem: My select1 is zero-based, so returning
        //
        // select(i + 1) does not help because then I get select1(1=5),
        // when there are only four 1s.

        // Ahhh! select0(0) to return 0 is just supposed to mean that
        // the first bucket (which select0(0) is supposed to return is made
        // as if it is there.
        //
        // So... by default, select1/0(0) returns
        //
        // Except.. the slides show select1(5) to return the 5th 1 and not
        // the 6th 1.

        // i == 0 works by default because select1(0) returns 0 by default,
        // despite there being no 1.

        // But.. removing the - i here or using the i does not help much,
        // because the i does not care about 0s in between (the distinction
        // between in-group 1s and between-group 0s).
        //
        // But.. how then do I choose u/upper_bits in a way that avoids
        // upper_bits and lower_bits overlapping, where one bit cares for
        // the other?
        //
        // Switched upper_bits and lower_bits.

        // question
        // Why is upper_part not 1 for 2 upper_bits?

        let upper_part: u64;

        if i == 0 {
            upper_part = self.upper.select1(i + 1)? - i;
        } else {
            // Working around peculiarity that select1 is 1-based,
            // while returning 0 for select1(0).
            //
            // Except: i + 1 for i == self.number_count goes over the limit.

            if i == self.numbers_count {
                upper_part = self.upper.select1(i + 1)? - i;
            } else {
                upper_part = self.upper.select1(i + 1)? - i;
            }
        }

        let mut lower_part = 0;

        for j in 0..self.lower_bits as u64 {
            // For acessing the last element, crashes.
            // It accessed self.lower[8], while the size of lower is 8.
            //
            // Maybe fix it by making access zero-based?
            // Or isn't it already?
            let bit = self.lower[(i * self.lower_bits + j) as usize];

            lower_part = lower_part | (if bit { 1 } else { 0 } << j);
        }

        println!(
            "access({}) - upper_part: {} lower_part: {}",
            i, upper_part, lower_part
        );

        return Ok((upper_part << (self.upper_bits) | lower_part) as u64);
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
        if v <= 0 {
            return 0;
        }

        return v - 1;
    }

    pub fn pred(&self, i: u64) -> Result<u64, MyError> {
        // This returns mostly the wrong numbers because the -i removes
        // the offset i, which currently with zero lower bits carries
        // the offset in the bucket.
        //
        // And we also do not take the bucket
        // into account: + bucket * pi_divisor.

        // Split into lower and upper.
        //
        // Quqestion: MSB is supposed to include zeroes and ones both, right?
        // Otherwise, lower becomes dynamic.
        let (lower, msb) = self.split(i);

        // Index in upper vector of the start of the bucket.
        //
        // +1 is necessary here because select0 is 1-based.
        // Except.. select0(2) returns the block after the one I want.
        let p = self.upper.select0(msb as u64)?;

        // Indexes up to and including the first in the bucket
        //
        // Because the prefix sum up to and including that one
        // is the index in the lower vector, we can start scanning using this.
        // p + 2 here because rank1 does not include the one directly pointing
        // to and so +1 would just include the zero from the group-start.
        let ith_in_original_numbers = self.upper.rank1(p + 2) - 1;

        // TODO: question
        // Why is ith_in_original_numbers = 1 here for i == 4 and msb == 1?
        // Ahh.. maybe -1 is not necessary for rank1?
        // Yes, -1 is necessary because rank1 counts all before and up to
        // p + 1, whereby p is the beginning of the block (the zero).

        // If ith is the last in the original numbers, then bucket is empty
        // anyway. Except.. if i-th is much earlier, then.. .
        if ith_in_original_numbers == self.numbers_count - 1 {
            println!("pred exit: last number - p: {} msb: {}", p, msb);
            return self.access(ith_in_original_numbers);
        }

        // p in upper is already false.
        //

        // Checks whether the number after p is false.
        // The idea is to check whether this is the same bucket.
        //
        // But.. do I really check that that way?
        // TODO:
        if self.upper.get(p + 1) == false {
            println!("pred exit: bucket empty");
            // We are in a higher bucket, so the bucket was empty, so we need to
            // take the last from a smaller bucket and return that.
            //
            //todo
            // Might be an issue that rank1 returns up to 4 while self.access
            // is zero-based.
            //
            // Except.. ith is the index in the original numbers, or is it not?
            // So.. access is one-based?
            // Except I subtract one except for zero.. maybe decrease only up
            // to 1.
            return self.access(self.upper.rank1(p + 1) - 1);
        }

        // Get the next bucket.
        //
        // But.. how do I handle there not being any bucket anymore?
        //
        // +1 here because select0 is 1-based.
        let next_bucket_p = self.upper.select0(msb as u64 + 1)?;

        // Indexes up to and including the last in the bucket
        //
        // No +1 or +2 here, because I want to just count until the end of the
        // previous bucket before next_bucket_p.
        let last_in_bucket_ith = self.upper.rank1(next_bucket_p) - 1;

        println!(
            "pred({}) - msb: {} p: {} ith: {} numbers_count: {} last_in_bucket_ith: {}",
            i, msb, p, ith_in_original_numbers, self.numbers_count, last_in_bucket_ith
        );

        // This bucket is non-empty.
        //
        // ith points to the first in that bucket.
        //
        // Two cases can happen:
        // a) I find bigger than lower in this bucket.
        // a1) ith is bigger: return self.access(ith-1)
        // a1.1) if ith-1 is zero, return self.access(0)
        // a1.2) if ith-1 is non-zero, return self.access(ith-1)
        // a2) other-than-ith is bigger: return self.access(bigger - 1)
        // b) I find none bigger than lower in this bucket, which means lower
        //    would be biggest because it is in that bucket. Return the last
        //    in this bucket.
        // c) If lower equals to lower_bits, return self.access(same).

        // BUt.. what about ith == lower?

        // Iterate in lower_vec by lower_bits.
        //
        // Can be sped up getting bucket boundaries and
        // halving each time for O(n log n).

        // Problem: This can go outside the bucket,
        // in which case the last of the bucket is supposed to be returned.
        for i in ith_in_original_numbers..=last_in_bucket_ith {
            let start = (i * self.lower_bits) as usize;
            let end = start + self.lower_bits as usize;

            let bits: &[bool] = &self.lower[start..end];
            let bits_number = Self::bits_to_u64(bits);

            println!(
                "i: {} start: {} end: {} bits: {:?} bits_number: {} lowr: {}",
                i, start, end, bits, bits_number, lower
            );

            if bits_number == lower {
                // c)
                println!("c)");
                return self.access(i);
            } else if bits_number > lower {
                // a)
                println!("a)");
                return self.access(Self::decrement_min_zero(i));
            }
        }

        println!(
            "lower: {} last_in_bucket_ith: {}",
            lower, last_in_bucket_ith
        );

        // b)
        return self.access(last_in_bucket_ith);
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

        for (i, query) in instance.queries.clone().iter().enumerate() {
            println!("Query nr {}: {}", i, query);
            let got = pd.pred(*query).unwrap();
            assert_eq!(want[i], got, "Query nr {}: {}", i, query);
        }

        // assert_eq!(want, got);
    }

    // Start benchmark
    benchmark(instance);
}

#[test]
fn testing_pd_access() {
    let pd = PD::new(&mut vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    let got: Vec<u64> = (0..10).map(|i| pd.access(i).unwrap()).collect();

    assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], got);

    // assert_eq!(0, pd.access(0).unwrap());
    // assert_eq!(1, pd.access(1).unwrap());
    // assert_eq!(2, pd.access(2).unwrap());
    // assert_eq!(3, pd.access(3).unwrap());
    // assert_eq!(4, pd.access(4).unwrap());
    // assert_eq!(5, pd.access(5).unwrap());
    // assert_eq!(6, pd.access(6).unwrap());
    // assert_eq!(7, pd.access(7).unwrap());
    // assert_eq!(8, pd.access(8).unwrap());
    // assert_eq!(9, pd.access(9).unwrap());
}

#[test]
fn testing_pd_test() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_4.txt");

    let want = vec![0, 0, 2, 2, 4, 4, 4, 7, 7, 7, 7];

    benchmark_and_check(path, Some(want));
}

#[test]
fn testing_pd_benchmark1() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_1.txt");

    benchmark_and_check(path, None);
}

#[test]
fn testing_xpd_benchmark2() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_2.txt");

    benchmark_and_check(path, None);
}

#[test]
fn testing_xpd_benchmark3() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_3.txt");

    benchmark_and_check(path, None);
}

#[test]
fn testing_pd_split() {
    let (mut lower, mut msb) = PD::split_with_bit_distribution(4, 2, 2);

    assert_eq!(0, lower);
    assert_eq!(1, msb);

    (lower, msb) = PD::split_with_bit_distribution(1, 2, 2);

    assert_eq!(1, lower);
    assert_eq!(0, msb);
}
