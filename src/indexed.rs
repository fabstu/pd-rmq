use std::time::Duration;

use malloc_size_of::MallocSizeOfOps;

#[derive(MallocSizeOf)]
pub struct IndexedBitVec {
    pub data: Vec<bool>,
}

pub fn report(algo: String, time: Duration, space: usize) {
    println!(
        "RESULT algo={} nameFabian_Sturm time={} space={}",
        algo,
        time.as_millis(),
        space
    );
}

impl IndexedBitVec {
    pub fn new(data: Vec<bool>) -> Self {
        Self { data }
    }

    pub fn pred(&self, n: usize) -> Option<usize> {
        if n == 0 || n > self.data.len() {
            return None;
        }
        for i in (0..n).rev() {
            if self.data[i] {
                return Some(i);
            }
        }
        None
    }

    pub fn succ(&self, n: usize) -> Option<usize> {
        if n >= self.data.len() {
            return None;
        }
        for i in n + 1..self.data.len() {
            if self.data[i] {
                return Some(i);
            }
        }
        None
    }
}
