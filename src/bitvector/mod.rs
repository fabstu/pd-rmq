mod rank1;
mod select1;

use std::error::Error;
use std::fmt;

pub use rank1::*;
pub use select1::*;

#[derive(MallocSizeOf, Clone)]
pub struct Bitvector {
    rank: Rank1,
    select0: Select1,
    select1: Select1,
    data: Vec<bool>,
}

#[derive(Debug, PartialEq)]
pub enum MyError {
    InvalidValue,
    //Select1GotZero,
    Select1NotEnough1s,
    Select1OutOfBounds,
    Select1SuperblockIndexOutOfBounds,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MyError::InvalidValue => f.write_str("Invalid value in vector"),
            // MyError::Select1GotZero => f.write_str("select1 got 0"),
            MyError::Select1NotEnough1s => f.write_str("select1 not enough 1s"),
            MyError::Select1OutOfBounds => {
                f.write_str("select1 out of bounds due to not enough 1s")
            }
            MyError::Select1SuperblockIndexOutOfBounds => {
                f.write_str("select1 superblock index out of bounds")
            }
        }
    }
}

impl Error for MyError {}

#[allow(dead_code)]
impl Bitvector {
    // Passes in a vector of 0s and 1s with lowest bits first.
    fn from_vec(vec: Vec<u8>) -> Result<Self, MyError> {
        if vec.iter().all(|b| *b == 1 || *b == 0) {
            Ok(Self::new(vec.iter().map(|b| *b == 1).collect()))
        } else {
            Err(MyError::InvalidValue)
        }
    }

    // Passes in a vector of 0s and 1s with lowest bits first.
    pub fn new(data: Vec<bool>) -> Self {
        let select1 = Select1::new(&data, true, false);
        let select0 = Select1::new(&data, false, false);

        println!("Select1-overall: {:?}", select1);
        //println!("Select0-overall: {:?}", select0);

        Self {
            rank: Rank1::new(&data),
            select0: select0,
            select1: select1,
            data: data,
        }
    }

    pub fn rank1(&self, i: u64) -> u64 {
        self.rank.rank1(&self.data[..], i)
    }
    pub fn rank0(&self, i: u64) -> u64 {
        self.rank.rank0(&self.data[..], i)
    }

    pub fn select0(&self, i: u64) -> Result<u64, MyError> {
        self.select0.select(&self.data[..], i)
    }

    pub fn select0_simple(&self, i: u64) -> Result<u64, MyError> {
        self.select0.select_simple(&self.data[..], i)
    }

    pub fn select0_naive(&self, i: u64) -> Result<u64, MyError> {
        self.select0.select_naive(&self.data[..], i)
    }

    pub fn select1(&self, i: u64) -> Result<u64, MyError> {
        println!("Select1: {}", i);
        self.select1.select(&self.data[..], i)
    }

    pub fn select1_simple(&self, i: u64) -> Result<u64, MyError> {
        self.select1.select_simple(&self.data[..], i)
    }

    pub fn select1_naive(&self, i: u64) -> Result<u64, MyError> {
        self.select1.select_naive(&self.data[..], i)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn testing_rank1() {
        let vec: Vec<u8> = vec![1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0];

        println!("vec: {:?}", vec);

        let bit_vector = Bitvector::from_vec(vec).unwrap();

        assert_eq!(bit_vector.rank1(0), 0);
        assert_eq!(bit_vector.rank1(1), 1);
        assert_eq!(bit_vector.rank1(2), 1);
        assert_eq!(bit_vector.rank1(3), 2);
        assert_eq!(bit_vector.rank1(4), 2);
        assert_eq!(bit_vector.rank1(5), 3);
        assert_eq!(bit_vector.rank1(6), 3);
        assert_eq!(bit_vector.rank1(7), 3);
        assert_eq!(bit_vector.rank1(8), 4);
        assert_eq!(bit_vector.rank1(9), 5);
        assert_eq!(bit_vector.rank1(10), 5);
        assert_eq!(bit_vector.rank1(11), 5);
        assert_eq!(bit_vector.rank1(12), 5);
        assert_eq!(bit_vector.rank1(13), 5);
        assert_eq!(bit_vector.rank1(14), 6);
        assert_eq!(bit_vector.rank1(15), 7);
    }

    #[test]
    fn testing_select1() {
        let vec: Vec<u8> = vec![1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0];

        println!("vec: {:?}", vec);

        let bit_vector = Bitvector::from_vec(vec).unwrap();

        testing_select1_variants("select1_simple", |i| bit_vector.select1_simple(i));
        testing_select1_variants("select1_naive", |i| bit_vector.select1_naive(i));
        testing_select1_variants("select1", |i| bit_vector.select1(i));
    }

    fn testing_select1_variants<F>(name: &'static str, select1: F)
    where
        F: Fn(u64) -> Result<u64, MyError>,
    {
        println!("testing select1: {}", name);

        // Except.. the documentation for Elias-Fano (pred)
        // assumes that select0(0) return 0.
        assert_eq!(select1(0).unwrap(), 0);
        assert_eq!(select1(1).unwrap(), 0);
        assert_eq!(select1(2).unwrap(), 2);
        assert_eq!(select1(3).unwrap(), 4);
        assert_eq!(select1(4).unwrap(), 7);
        assert_eq!(select1(5).unwrap(), 8);
        assert_eq!(select1(6).unwrap(), 13);
        assert_eq!(select1(7).unwrap(), 14);
        // Only 7 1s in the bitvector.
        assert_eq!(select1(8).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(9).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(10).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(11).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(12).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(13).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(14).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(15).unwrap_err(), MyError::Select1NotEnough1s);
        // Out of bounds of the bitvector, can never be that many 1s.
        assert_eq!(select1(16).unwrap_err(), MyError::Select1OutOfBounds);
    }
}

fn u64_to_vec_bool(n: u64, bit_size: u64) -> Vec<bool> {
    // Find out how many bits are required to represent the number.

    // Create the vector with each bit encoded as a bool.
    let mut vec = Vec::with_capacity(bit_size as usize);

    // Reverse here because wanna store from lowest to highest
    // significant bit.
    for i in (0..bit_size).rev() {
        let bit = (n >> (bit_size - 1 - i)) & 1;
        vec.push(bit == 1);
    }

    vec
}
