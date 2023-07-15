use super::Bitvector;

mod select1_naive;

use super::MyError;
pub use select1_naive::Select1Naive;

#[derive(MallocSizeOf, Clone)]
pub struct Select1 {
    // Whether it does select1 or select0.
    is1: bool,

    //todo("implement select1")
    // For each superblock this naive or the sub-blocks with naive or lookup table.
    // For size < log^4 n sub-block:
    // select1Naive: Select1Naive,
    // select1_lookup_table: HashMap<u64, u64>,

    // Each superblock stores b #1s up to the start of the superblock.
    // This array contains the index the superblock
    // starts at.
    // Or ends at?
    //
    // Well.. the slides noted floor(i/b) - 1 for the superblock index.
    // So.. maybe that means the end of the previous superblock.
    // So.. superstep_end_index[last-superblock] is never called.
    //
    // Although using end_index and -1 means I have to special-case
    // access to the 1st superblock. Or return 0 if a negative index is
    // accessed.
    //
    // Not sure whether superstep_1s[0] is 0 or the size of the 1st superblock.
    // If it is the index of the 1st superblock, then have to
    superstep_end_index: Vec<u64>,
}

impl Select1 {
    pub fn new(data: &Vec<bool>, is1: bool) -> Self {
        Self {
            is1,
            superstep_end_index: Vec::new(),
        }
    }

    fn is_one(&self, value: bool) -> bool {
        if self.is1 {
            return value;
        } else {
            return !value;
        }
    }

    pub fn select(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        let mut count = 0;
        for j in 0..data.len() as u64 {
            if self.is_one(data[j as usize]) {
                count += 1;
            }
            if count == i {
                return Ok(j);
            }
        }

        return Err(MyError::Select1NotEnough1s);
    }

    pub fn select_simple(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        let mut count = 0;
        for j in 0..data.len() as u64 {
            if self.is_one(data[j as usize]) {
                count += 1;
            }
            if count == i {
                return Ok(j);
            }
        }

        return Err(MyError::Select1NotEnough1s);
    }

    pub fn select_naive(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        // Trying naive for whole bitvector.
        let naive = Select1Naive::new(&data[..]);

        return naive.select1(i);
    }

    pub fn select_simple_old(&self, data: &[bool], i: u64) -> u64 {
        let mut count = 0;
        for j in 0..data.len() as u64 {
            if self.is_one(data[j as usize]) {
                count += 1;
            }
            if count == i {
                return count;
            }
        }
        panic!("select out of bounds");
    }
}
