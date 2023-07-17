use std::collections::HashSet;

pub struct SparseBitVec {
    set: HashSet<usize>,
}

impl SparseBitVec {
    // Creates a new SparseBitVec
    pub fn new() -> SparseBitVec {
        SparseBitVec {
            set: HashSet::new(),
        }
    }

    pub fn from_vec(vec: Vec<bool>) -> SparseBitVec {
        let mut set = HashSet::new();
        for (i, b) in vec.iter().enumerate() {
            if *b {
                set.insert(i);
            }
        }
        SparseBitVec { set: set }
    }

    // Sets the bit at index to 1
    pub fn insert(&mut self, index: usize) {
        self.set.insert(index);
    }

    // Sets the bit at index to 0
    pub fn remove(&mut self, index: usize) {
        self.set.remove(&index);
    }

    // Checks whether the bit at index is 1
    pub fn contains(&self, index: usize) -> bool {
        self.set.contains(&index)
    }

    // Gets a vector of bools for the bits in the given range
    pub fn get_range(&self, start: usize, end: usize) -> Vec<bool> {
        (start..end).map(|i| self.contains(i)).collect()
    }
}

impl IntoIterator for SparseBitVec {
    type Item = usize;
    type IntoIter = std::collections::hash_set::IntoIter<usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.into_iter()
    }
}
