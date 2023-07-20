use super::RMQError;
use super::RMQ;

#[derive(MallocSizeOf, Clone)]
pub struct RMQNaiveFast {
    naive: Vec<Vec<usize>>,
}

impl RMQNaiveFast {
    pub fn new(numbers: Vec<u64>) -> Self {
        let n = numbers.len();

        // O(u^2). Could do u log n using consecutively less space for "to".
        //
        let mut naive = vec![vec![std::usize::MAX; n as usize]; n as usize];

        for i in 0..n {
            // Minimum to itself.
            naive[i][i] = i;

            for j in i + 1..n {
                // We only grow, so with each step we take on a new number.
                //
                // If the new number is smaller than the current, then record
                // it. Else keep the current minimum.
                if numbers[naive[i][j - 1]] < numbers[j] {
                    naive[i][j] = naive[i][j - 1];
                } else {
                    naive[i][j] = j;
                }
            }
        }

        return Self { naive: naive };
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        // Stay in range.
        assert!(from < self.naive.len());
        assert!(to < self.naive.len());

        return Ok(self.naive[from][to]);
    }
}

impl RMQ for RMQNaiveFast {
    fn new(numbers: Vec<u64>) -> Self {
        RMQNaiveFast::new(numbers)
    }

    fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        RMQNaiveFast::range_minimum_query(self, from, to)
    }
}
