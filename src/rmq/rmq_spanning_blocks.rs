use super::RMQError;
use super::RMQ;

use super::rmq_sparse::RMQSparse;

#[derive(MallocSizeOf, Clone)]
pub struct RMQSpanningBlocks {
    //2:
    block_minimum: Vec<u64>,
    // Allows range minimum query over whole blocks.
    block_minimum_sparse: RMQSparse,
    // Wanna return position of minimum, not minimum itself.
    block_minimum_position_in_block: Vec<usize>,
}

impl RMQSpanningBlocks {
    pub fn new(numbers: Vec<u64>) -> Self {
        let n = (numbers.len() - 1) as f64;

        let block_size = (n.log2() / 4.0).ceil();
        let block_count = (n / block_size).ceil();

        // Query types:
        // 1) Zwei Teilblöcke + mehrere Blöcke
        // 2) umfasst ganze Blöcke: Blockgrenze zu Blockgrenze
        // 3) 1-2 Teilblöcke: Innerhalb eines Blocks oder eine grenze kreuzend.

        //2:
        // Stores minimum per whole block.
        let block_minimum = vec![0u64; block_count as usize];

        // Verwende n log n-DS Sparse Table für B.
        let block_minimum_sparse = RMQSparse::new(block_minimum.clone());

        let block_minimum_position_in_block = vec![0usize; block_count as usize];

        return Self {
            block_minimum: block_minimum,
            block_minimum_sparse: block_minimum_sparse,
            block_minimum_position_in_block: block_minimum_position_in_block,
        };
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
        panic!("not implemented yet");

        // Stay in range.
        // assert!(from < self.naive.len());
        // assert!(to < self.naive.len());

        // return Ok(self.naive[from][to]);
    }
}

impl RMQ for RMQSpanningBlocks {
    fn new(numbers: Vec<u64>) -> Self {
        RMQSpanningBlocks::new(numbers)
    }

    fn range_minimum_query(&self, from: usize, to: usize) -> Result<u64, RMQError> {
        RMQSpanningBlocks::range_minimum_query(self, from, to)
    }
}
