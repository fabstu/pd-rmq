use super::Bitvector;

mod select1_naive;

use super::MyError;
pub use select1_naive::Select1Naive;

impl Bitvector {
    pub fn select1(&self, i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= self.data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return Ok(j);
            }
        }

        return Err(MyError::Select1NotEnough1s);
    }

    pub fn select1_simple(&self, i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= self.data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return Ok(j);
            }
        }

        return Err(MyError::Select1NotEnough1s);
    }

    pub fn select1_naive(&self, i: u64) -> Result<u64, MyError> {
        // Trying naive for whole bitvector.
        let naive = Select1Naive::new(&self.data[..]);

        return naive.select1(i);
    }

    pub fn select0(&self, i: u64) -> u64 {
        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return count;
            }
        }
        panic!("select out of bounds");
    }
}
