use crate::debug::DEBUG;

use super::RMQError;
use super::RMQ;

#[derive(MallocSizeOf, Clone)]
pub struct RMQSparse {
    m: Vec<Vec<usize>>,
    pub numbers: Vec<u64>,
}

impl RMQSparse {
    pub fn new(numbers: Vec<u64>) -> Self {
        let n = numbers.len();
        let k = (n as f64).log2() as usize;

        // Holds for m[i][j] the range minimum from i to 2^j
        let mut m = vec![vec![0usize; k + 1]; n];

        for i in 0..n {
            // RMQ to itself is the element itself.
            m[i][0] = i as usize;
        }

        let mut j = 1;
        while 1 << j < n {
            // Compute minimum of every range of length 2^j
            let mut i = 0;
            while i + (1 << j) - 1 < n {
                if numbers[m[i][j - 1]] < numbers[m[i + (1 << (j - 1))][j - 1]] {
                    m[i][j] = m[i][j - 1];
                } else {
                    m[i][j] = m[i + (1 << (j - 1))][j - 1];
                }

                i += 1;
            }

            j += 1;
        }

        Self {
            m: m,
            numbers: numbers,
        }
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        if DEBUG {
            println!("Sparse query: from: {}, to: {}", from, to);
        }

        let l = ((to + 1 - from) as f64).log2().floor() as usize;

        // Overlapping ranges of power-of-two length.
        // No issue, because we are looking for the minimum.
        let m1 = self.m[from][l] as usize;
        let m2 = self.m[to - (1 << l)][l] as usize;

        if DEBUG {
            println!(
            "Sparse query: from: {}, to: {}, l: {}, m1_index: {}, m2_index: {}, m1_value: {}, m2_value: {}",
            from, to, l, m1, m2, self.numbers[m1], self.numbers[m2]
        );
        }

        if self.numbers[m1] < self.numbers[m2] {
            return Ok(m1);
        }

        return Ok(m2);
    }

    // pub fn new(numbers: Vec<u64>) -> Self {
    //     let n = numbers.len();
    //     let k = (n as f64).log2() as usize;

    //     // Holds for m[i][j] the range minimum from i to 2^j
    //     let mut m = vec![vec![0usize; k + 1]; n];

    //     for i in 0..n {
    //         // RMQ to itself is the element itself.
    //         m[i][0] = numbers[i] as usize;
    //     }

    //     let mut j = 1;
    //     while 1 << j < n {
    //         // Compute minimum of every range of length 2^j
    //         let mut i = 0;
    //         while i + (1 << j) - 1 < n {
    //             m[i][j] = std::cmp::min(m[i][j - 1], m[i + (1 << (j - 1))][j - 1]);

    //             i += 1;
    //         }

    //         j += 1;
    //     }

    //     Self {
    //         m: m,
    //         numbers: numbers,
    //     }
    // }

    // pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
    //     let l = ((to - from + 1) as f64).log2().floor() as usize;

    //     // Overlapping ranges of power-of-two length.
    //     // No issue, because we are looking for the minimum.
    //     let m1 = self.m[from][l] as u64;
    //     let m2 = self.m[to - (1 << l)][l] as u64;

    //     return Ok(std::cmp::min(m1, m2));
    // }
}

impl RMQ for RMQSparse {
    fn new(numbers: Vec<u64>) -> Self {
        RMQSparse::new(numbers)
    }

    fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        RMQSparse::range_minimum_query(self, from, to)
    }
}
