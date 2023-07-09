use std::time::Duration;

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

    pub fn rank0(&self, i: u64) -> usize {
        let mut count = 0;
        for j in 0..i {
            if self.data[j as usize] {
                count += 1;
            }
        }
        count
    }

    pub fn select0(&self, i: u64) -> u64 {
        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return j;
            }
        }
        panic!("select out of bounds");
    }

    pub fn rank1(&self, i: u64) -> u64 {
        let mut count = 0;
        for j in 0..i {
            if !self.data[j as usize] {
                count += 1;
            }
        }
        count
    }

    pub fn select1(&self, i: u64) -> u64 {
        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if !self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return j;
            }
        }
        panic!("select out of bounds");
    }
}
