use std::collections::HashMap;

mod select1_naive;

use super::MyError;
pub use select1_naive::Select1Naive;

#[derive(MallocSizeOf, Clone)]
pub struct Select1 {
    // Whether it does select1 or select0.
    is1: bool,

    // Overall number of zeroes/ones.
    k: u32,
    // Number of zeroes/ones per superblock.
    b: u32,

    //todo("implement select1")
    // For each superblock this naive or the sub-blocks with naive or lookup table.
    // For size < log^4 n sub-block:
    // select1Naive: Select1Naive,
    // select1_lookup_table: HashMap<u64, u64>,

    // Each superblock stores b #1s up to the start of the superblock.
    // This array contains the index the superblock
    // starts at.
    // Or ends at?
    //
    // Well.. the slides noted floor(i/b) - 1 for the superblock index.
    // So.. maybe that means the end of the previous superblock.
    // So.. superstep_end_index[last-superblock] is never called.
    //
    // Although using end_index and -1 means I have to special-case
    // access to the 1st superblock. Or return 0 if a negative index is
    // accessed.
    //
    // Not sure whether superstep_1s[0] is 0 or the size of the 1st superblock.
    // If it is the index of the 1st superblock, then have to
    superblock_end_index: Vec<u64>,

    in_superblock: Vec<InSuperblockSelect>,

    lookup_table: HashMap<Vec<bool>, HashMap<u64, u64>>,
}

// Need two different implementations for superblock and subblock.
// Why: Because one offers naive/subblock for its block while two
// offers naive/lookup table for its block.
//
// So.. the shared functionality is superblock_end_index, while the strategy in
// how in_superblock is created is different.
//
// So.. can just follow strategy laid out while implementing the difference
// in the constructor?
//
// - Makes it super simple.
// - Still allows constructing recursively, but I can just avoid that by not
//   building recursively, or offering a switch that is turned off on th 1st
//   recursion. The switch then also decides which of the two strategy-creations
//   are used.
//
#[derive(MallocSizeOf, Clone)]
enum InSuperblockSelect {
    Naive(Select1Naive), // Allowed on 1st and 2nd level.
    Subblock(Select1),   // Allowed on 1st level
    LookupTable,         // Allowed on 2nd level.
}

impl Select1 {
    pub fn new(data: &Vec<bool>, is1: bool) -> Self {
        let n = data.len();
        // Sum of all zeroes/ones.
        let k = data.iter().filter(|v| **v == is1).count() as u32;
        // Number of zeroes/ones per superblock.
        //
        // Not sure whether floor or ceil or staying float.
        let b = (n as f64).log2().powf(2.0).floor() as u32;

        // Number of superblocks because we have k zeroes/ones
        // which are split of into blocks of b, leaving the resulting #blocks.
        let number_of_superblocks = k / b;

        // Prefix-sum to have #1s for each superblock.

        let mut superblock_end_index: Vec<u32> = Vec::new();
        let mut count = 0;

        for (i, val) in data.iter().enumerate() {
            // is1 == true means methods act as select1, is1 == false is for
            // select0.
            if *val != is1 {
                continue;
            }

            count += 1;

            if count % b == 0 {
                // Question is.. do i do this before or after the count += 1?
                // a) Before because this is the last zero, meaning the last
                //    index contained in the superblock.
                //    But.. Doesn't this then point to the end of this
                //    superblock?
                //    What does this do to select1/0?
                //
                //    This would the end of one and the start of the new
                //    superblock do not overlap. This makes sense anyway, right?
                //    There is no guarantee that the next 1/0 is the start of
                //    the next block. All the 0s and 1s after are part of the
                //    new block and so handled by that naive/sub-block/lookup
                //    table.
                // b) After: No good.
                superblock_end_index.push(i as u32);
            }

            // Fine that this cuts off the mantisse?
            //
            // Anyway, this here is useless since we determine superblock_index
            // by #1s/#0s using count.
            //let superblock_index = i as u32 / b;

            //
        }

        // for superblock_index in 0..numberOfSuperblocks {

        // }

        Self {
            is1: is1,
            k: k,
            b: b,
            in_superblock: in_superblock,
            superblock_end_index: Vec::new(),
            lookup_table: lookup_table,
        }
    }

    fn is_one(&self, value: bool) -> bool {
        if self.is1 {
            return value;
        } else {
            return !value;
        }
    }

    pub fn select(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        if i > self.k as u64 {
            return Err(MyError::Select1NotEnough1s);
        }

        // Cuts of mantisse, meaning automatic floor() without the rounding up.
        let superblock_number = i / self.b as u64;

        // Not sure whether superblock_number can be
        // equal to superblock_end_index.len().
        //
        // But restricting it to < ensures the problems is immediately obivous
        // while ensuring I can access the current blocks' end.
        if superblock_number >= self.superblock_end_index.len() as u64 {
            return Err(MyError::Select1SuperblockIndexOutOfBounds);
        }

        let this_superblock_start_index: u64;

        // Handle pointing to end of previous superblock when there is no
        // previous superblock.
        if superblock_number > 0 {
            // One after the end of the previous superblock if there was a
            // previous superblock, because the last superblock ended with an
            // isOne.
            let previous_superblock_end_index =
                self.superblock_end_index[(superblock_number - 1) as usize];

            this_superblock_start_index = previous_superblock_end_index + 1;
        } else {
            // If there is no previous superblock, then we start pointing
            // to the beginning of the block.
            this_superblock_start_index = 0;
        }

        // Add in-superblock depending on naive or  sub-superblocks with (naive or lookup table).

        let this_superblock_end_index: usize =
            self.superblock_end_index[superblock_number as usize] as usize;

        let in_block_offset: u64;

        match self.in_superblock[superblock_number as usize] {
            InSuperblockSelect::Naive(ref naive) => {
                // What do I pass as i here? 0 == beginning of block.
                // But if it returns 0,
                in_block_offset = naive.select(i)?;
            }
            InSuperblockSelect::Subblock(ref subblock) => {
                // What data to pass here?
                in_block_offset = subblock.select(
                    &data[this_superblock_start_index as usize..this_superblock_end_index],
                    i,
                )?;
            }
            InSuperblockSelect::LookupTable => {
                // What data to pass here?
                // Need block beginning to end.
                // And if its the last block, beginning of block to end of
                // global data.
                in_block_offset = self.lookup_table_select(
                    &data[this_superblock_start_index as usize..this_superblock_end_index],
                    i,
                );
            }
        }

        return Ok(this_superblock_start_index + in_block_offset);
    }

    fn lookup_table_select(&self, data: &[bool], i: u64) -> u64 {
        self.lookup_table[data][&i]
    }

    pub fn select_simple(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        let mut count = 0;
        for j in 0..data.len() as u64 {
            if self.is_one(data[j as usize]) {
                count += 1;
            }
            if count == i {
                return Ok(j);
            }
        }

        return Err(MyError::Select1NotEnough1s);
    }

    pub fn select_naive(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        // Trying naive for whole bitvector.
        let naive = Select1Naive::new(&data[..], self.is1);

        return naive.select(i);
    }

    #[allow(dead_code)]
    pub fn select_simple_old(&self, data: &[bool], i: u64) -> u64 {
        let mut count = 0;
        for j in 0..data.len() as u64 {
            if self.is_one(data[j as usize]) {
                count += 1;
            }
            if count == i {
                return count;
            }
        }
        panic!("select out of bounds");
    }
}
