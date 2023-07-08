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

// TODO: Make this performant like shown in lecture.
impl IndexedBitVec {
    pub fn new(data: Vec<bool>) -> Self {
        Self { data }
    }

    pub fn rank(&self, i: usize) -> usize {
        let mut count = 0;
        for j in 0..i {
            if self.data[j] {
                count += 1;
            }
        }
        count
    }

    pub fn select(&self, i: usize) -> usize {
        let mut count = 0;
        for j in 0..self.data.len() {
            if self.data[j] {
                count += 1;
            }
            if count == i {
                return j;
            }
        }
        panic!("select out of bounds");
    }
}
