use std::collections::HashMap;

mod select1_naive;

use super::u64_to_vec_bool;
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
        }

        let in_superblock: Vec<InSuperblockSelect> = Vec::new();

        // Insert for each superblock its way of getting the index inside
        // in constant time.

        let maximum_block_size_in_bits = 0;

        let mut select_lookup_table: HashMap<Vec<bool>, HashMap<u64, u64>> = HashMap::new();

        // All possible values for block_size.
        for i in 0..2u64.pow((maximum_block_size_in_bits) as u32) {
            // Get block bitvector pattern.
            let block = u64_to_vec_bool(i, maximum_block_size_in_bits);
            let mut block_lookups: HashMap<u64, u64> = HashMap::new();

            let number_of_ones_zeroes = block.iter().filter(|v| **v == is1).count() as u64;

            // Go up to numberOfOnesOrZeroes + 1 because we want to include
            // the last found match as well. We start at 1 after all and
            // with ..X, X is excluded.
            for lookup in 0..number_of_ones_zeroes + 1 {
                // How do I get #1s in i up to lookup without calcing rank1 for
                // each lookup manually?
                //
                // a) flip each bit (block_bitsize is bit-count) manually, and
                //    update #1s on each flip.

                let mut select1: u64 = 0;

                if lookup == 0 {
                    select1 = 0;
                } else {
                    // Calculate select1 for this block and lookup.
                    //
                    // a) flip each bit manually, and adapt #1s on each flip.
                    // b) Slow: Iterate over block and up to found
                    //    lookup #1s/#0s.
                    //
                    //    Except.. size < log^4 n is super small, so might be
                    //    insignificant anyway.
                    let mut count = 0;
                    let mut found = false;

                    for index in 0..block.len() {
                        if block[index] != is1 {
                            continue;
                        }

                        count += 1;

                        if count == lookup {
                            select1 = index as u64;
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        panic!(
                            "Could not find select1 for block: {:#?} lookup: {}",
                            block, lookup
                        );
                    }
                }

                println!(
                    "ins select1_lookup - max block size: {} i: {} block: {:#?} lookup: {} rank1: {}",
                    maximum_block_size_in_bits, i, block, lookup, select1,
                );

                block_lookups.insert(lookup, select1);
            }

            select_lookup_table.insert(block, block_lookups);
        }

        // How do I know the maximum #bits I need for the lookup table?
        // a) b: #bits per superblock.
        // b) size < log^4 n: Only using lookup table in this case.
        // c) Observe the maximum block size from previous iteration.

        Self {
            is1: is1,
            k: k,
            b: b,
            in_superblock: in_superblock,
            superblock_end_index: Vec::new(),
            lookup_table: select_lookup_table,
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
        if superblock_number > self.superblock_end_index.len() as u64 {
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

        let this_superblock_end_index: usize;

        if superblock_number as usize == self.superblock_end_index.len() {
            // The last superblock does not have a full block.
            this_superblock_end_index = data.len() - 1;
        } else {
            // Read this non-last superblock's end index.
            this_superblock_end_index =
                self.superblock_end_index[superblock_number as usize] as usize;
        }

        let in_block_offset: u64;

        let i_excluding_previous_superblocks: u64;

        if superblock_number == 0 {
            // No previous superblock, so all i ones/zeroes are to be found
            // in this block.
            i_excluding_previous_superblocks = i;
        } else {
            // Only count the previous superblocks.
            i_excluding_previous_superblocks = i - ((superblock_number - 1) * self.b as u64);
        }

        match self.in_superblock[superblock_number as usize] {
            InSuperblockSelect::Naive(ref naive) => {
                // What do I pass as i here? 0 == beginning of block.
                // But if it returns 0,
                in_block_offset = naive.select(i_excluding_previous_superblocks)?;
            }
            InSuperblockSelect::Subblock(ref subblock) => {
                // What data to pass here?
                in_block_offset = subblock.select(
                    &data[this_superblock_start_index as usize..this_superblock_end_index],
                    i_excluding_previous_superblocks,
                )?;
            }
            InSuperblockSelect::LookupTable => {
                // What data to pass here?
                // Need block beginning to end.
                // And if its the last block, beginning of block to end of
                // global data.
                in_block_offset = self.lookup_table_select(
                    &data[this_superblock_start_index as usize..this_superblock_end_index],
                    i_excluding_previous_superblocks,
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
