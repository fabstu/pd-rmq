use super::RMQError;
use super::RMQ;

#[derive(MallocSizeOf, Clone)]
pub struct RMQNaiveSlow {
    naive: Vec<Vec<u64>>,
}

impl RMQNaiveSlow {
    pub fn new(numbers: &Vec<u64>) -> Self {
        let n = numbers.len();

        // O(u^2). Could do u log n using consecutively less space for "to".
        //
        let mut naive = vec![vec![std::u64::MAX; n as usize]; n as usize];

        for i in 0..n {
            // Minimum to itself.
            naive[i][i] = numbers[i];

            for j in i + 1..n {
                // We only grow, so with each step we take on a new number.
                //
                // If the new number is smaller than the current, then record
                // it. Else keep the current minimum.
                naive[i as usize][j as usize] = std::cmp::min(naive[i][j - 1], numbers[j]);
            }
        }

        return Self { naive: naive };
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
        // Stay in range.
        assert!(from < self.naive.len());
        assert!(to < self.naive.len());

        return Ok(self.naive[from][to]);
    }
}

impl RMQ for RMQNaiveSlow {
    fn new(numbers: &Vec<u64>) -> Self {
        RMQNaiveSlow::new(numbers)
    }

    fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
        RMQNaiveSlow::range_minimum_query(self, from, to)
    }
}
