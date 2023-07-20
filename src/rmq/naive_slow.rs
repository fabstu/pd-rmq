use crate::debug::DEBUG;

use super::RMQError;
use super::RMQ;

#[derive(MallocSizeOf, Clone)]
pub struct RMQNaiveSlow {
    numbers: Vec<u64>,
}

impl RMQNaiveSlow {
    pub fn new(numbers: Vec<u64>) -> Self {
        return Self { numbers: numbers };
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        if from >= self.numbers.len() || to >= self.numbers.len() {
            return Err(RMQError::OutOfRange);
        }

        let mut min_index = 0;
        let mut min_value = std::u64::MAX;

        for i in from..=to {
            if self.numbers[i] < min_value {
                min_index = i;
                min_value = self.numbers[i];
            }
        }

        // let minimum = self.numbers[from..=to]
        //     .iter()
        //     //.min()
        //     .enumerate()
        //     .min_by(|(_, a), (_, b)| a.cmp(b))
        //     .unwrap();

        // let min_index = minimum.0;

        assert!(
            min_index >= from && min_index <= to,
            "min index {} not in range {}..={}",
            min_index,
            from,
            to
        );

        if DEBUG {
            println!(
                "RMQNaiveSlow::range_minimum_query({}, {}) = {} min_value={}",
                from, to, min_index, min_value
            );
        }

        Ok(min_index)
    }
}

impl RMQ for RMQNaiveSlow {
    fn new(numbers: Vec<u64>) -> Self {
        RMQNaiveSlow::new(numbers)
    }

    fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        RMQNaiveSlow::range_minimum_query(self, from, to)
    }
}
