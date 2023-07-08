pub struct IndexedBitVec {
    data: Vec<bool>,
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
