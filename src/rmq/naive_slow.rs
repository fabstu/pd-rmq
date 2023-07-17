use super::RMQError;
use super::RMQ;

#[derive(MallocSizeOf, Clone)]
pub struct RMQNaiveSlow {
    numbers: Vec<u64>,
}

impl RMQNaiveSlow {
    pub fn new(numbers: Vec<u64>) -> Self {
        return Self {
            numbers: numbers.clone(),
        };
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
        if from >= self.numbers.len() || to >= self.numbers.len() {
            return Err(RMQError::OutOfRange);
        }

        let minimum = self.numbers[from..=to].iter().min().cloned().unwrap();
        Ok(minimum)
    }
}

impl RMQ for RMQNaiveSlow {
    fn new(numbers: Vec<u64>) -> Self {
        RMQNaiveSlow::new(numbers)
    }

    fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
        RMQNaiveSlow::range_minimum_query(self, from, to)
    }
}
