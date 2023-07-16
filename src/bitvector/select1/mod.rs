use std::collections::HashMap;

mod select1_naive;

use super::u64_to_vec_bool;
use super::MyError;
pub use select1_naive::Select1Naive;

#[derive(MallocSizeOf, Clone, Debug)]
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
    lookup_table_block_size: u32,

    is_subblock: bool,
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
#[derive(MallocSizeOf, Clone, Debug)]
enum InSuperblockSelect {
    Naive(Select1Naive), // Allowed on 1st and 2nd level.
    Subblock(Select1),   // Allowed on 1st level
    LookupTable,         // Allowed on 2nd level.
}

impl Select1 {
    //pub fn new(data: &[bool], is1: bool, is_subblock: bool) -> Self {

    pub fn new(data: &[bool], is1: bool, is_subblock: bool) -> Self {
        let n = data.len();
        // Sum of all zeroes/ones.
        let k = data.iter().filter(|v| **v == is1).count() as u32;
        // Number of zeroes/ones per superblock.
        //
        // Not sure whether floor or ceil or staying float.
        let b: u32;

        if n <= 1 {
            b = 1;
        } else {
            if !is_subblock {
                // let log_n = (n as f64).log2();
                // b = (log_n * log_n) as u32;
                //b = (n as f64).log2().powf(2.0).floor() as u32;
                b = (n as f64).log2().floor() as u32;
            } else {
                // Calculate b' as Wurzel(log2 n) instead when in the subblock.
                b = (n as f64).log2().sqrt().floor() as u32;
            }
        }

        println!("{} n={}, k={}, b={}", space(is1, is_subblock), n, k, b);

        // Number of superblocks because we have k zeroes/ones
        // which are split of into blocks of b, leaving the resulting #blocks.
        //let number_of_superblocks = ceil(b / k);

        // Prefix-sum to have #1s for each superblock.
        let mut superblock_end_index: Vec<u64> = Vec::new();

        // Insert for each superblock its way of getting the index inside
        // in constant time.
        let mut in_superblock: Vec<InSuperblockSelect> = Vec::new();

        assert_ne!(b, 0, "b must not be 0. n={}", n);

        let mut count = 0;

        let mut superblock_start: usize = 0;
        let mut superblock_end: usize = 0;
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
                //
                // But..why is no end_index added?
                superblock_end_index.push(i as u64);

                superblock_end = i;

                // Add to in_superblock.
                in_superblock.push(Self::in_superblock_for(
                    data,
                    n,
                    is1,
                    superblock_start,
                    superblock_end,
                    is_subblock,
                ));

                // Next superblock starts at next index.
                superblock_start = i + 1;
            }
        }

        // if n == 1 {
        //     // None was and added because b = 1.
        // }

        if superblock_end < data.len() - 1 {
            println!("{} Last: ", space(is1, is_subblock));

            // Problem: In my case the third is skipped.

            // Insert last superblock that was not finished.
            in_superblock.push(Self::in_superblock_for(
                data,
                n,
                is1,
                // Problem: if superblock_end was not added to again, then
                // superblock_end - 1 is smaller than superblock_start.
                // So:
                // a) Special-case this somehow.
                // b) Think whether the -1 makes sense anyway.
                // c) Set both to the same value when switching to new superblock
                //    and avoid the -1.
                //
                // Tried c), but then superblock_end is too big here when doing
                // last.
                //
                // d) Only add the last when superblock_start <= superblock_end.
                //    The idea is that when the new superblock was not started,
                //    then there is no need to add the non-existent superblock.
                //
                // Problem now is that I made start = end + 1 and end = end - 1
                superblock_start,
                // Last superblock goes up to the end of the data.
                // -1 because the end is included.
                data.len() - 1,
                is_subblock,
            ));
        }

        println!(
            "{} Added superblock_end_indexes b: {} superblock_end_indexes: {:?} ",
            space(is1, is_subblock),
            b,
            superblock_end_index
        );

        // As in the slides.
        let maximum_block_size_in_bits: u64 = (n as f64).log2().ceil() as u64;

        let mut select_lookup_table: HashMap<Vec<bool>, HashMap<u64, u64>> = HashMap::new();

        // All possible values for block_size.
        //
        // This might be too much.. .
        for i in 0..2u64.pow((maximum_block_size_in_bits) as u32) {
            // Get block bitvector pattern.
            //
            // What is the bit-size of the blocks?
            // One case I had was where the input-blockrange was 2bit
            // but the lookup_table blocksize was 3bit, making the app crash.
            //
            // So.. -1 for now.
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

                /*
                println!(
                    "ins select1_lookup - max block size: {} i: {} block: {:#?} lookup: {} rank1: {}",
                    maximum_block_size_in_bits, i, block, lookup, select1,
                );
                */

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
            superblock_end_index: superblock_end_index,
            lookup_table: select_lookup_table,
            lookup_table_block_size: maximum_block_size_in_bits as u32,
            is_subblock: is_subblock,
        }
    }

    fn is_one(&self, value: bool) -> bool {
        if self.is1 {
            return value;
        } else {
            return !value;
        }
    }

    pub fn selectWithBoundaryCheck(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        // Except: i == 1 just means that I want a single 1,
        // which can with len == 1 mean to return 0, or 1.
        // self.
        // Problem: For a sub-select, it is not necessarily an out-of-bounds
        // for i to be == data.len because prior superblocks already consume
        // more non-matching size.
        //
        // a) Only do this check once on the main level. Recursive calls
        //    call the down-specced check-version.
        // b) Return 0 on the sub-level here if i >= self.k.
        //    This works because self.k only == data.len() if all match.
        if i >= data.len() as u64 {
            println!(
                "{} Accesing i={} in data of len={} data: {:#?}",
                space(self.is1, self.is_subblock),
                i,
                data.len(),
                data
            );
            return Err(MyError::Select1OutOfBounds);
        }

        return self.select(data, i);
    }

    pub fn select(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Ok(0);
        }

        if i > self.k as u64 {
            return Err(MyError::Select1NotEnough1s);
        }

        // Cuts of mantisse, meaning automatic floor() without the rounding up.
        let superblock_number = i / self.b as u64;

        // Problem: Returns superblock_number=1, naive gets i=3 unchanged.
        // When I change it to i=0 by not subtracting the superblock,
        // then I pass in i=0 to Naive, which does not accept i==0.
        // So..
        // a) [x] Drop Select1GotZero and just return 0. <- simpler
        // b)     Special-case non-outer returns to return 0 instead of Select1GotZero.

        // Not sure whether superblock_number can be
        // equal to superblock_end_index.len().
        //
        // But restricting it to < ensures the problems is immediately obivous
        // while ensuring I can access the current blocks' end.
        if superblock_number > self.superblock_end_index.len() as u64 {
            println!(
                "i: {} b: {} superblock_number: {} superblock_end_index.len(): {}",
                i,
                self.b,
                superblock_number,
                self.superblock_end_index.len()
            );
            return Err(MyError::Select1SuperblockIndexOutOfBounds);
        }

        let mut previous_superblock_end_index: u64 = 0;
        let this_superblock_start_index: u64;

        // Handle pointing to end of previous superblock when there is no
        // previous superblock.
        if superblock_number > 0 {
            // One after the end of the previous superblock if there was a
            // previous superblock, because the last superblock ended with an
            // isOne.
            previous_superblock_end_index =
                self.superblock_end_index[(superblock_number - 1) as usize];

            this_superblock_start_index = previous_superblock_end_index + 1;
        } else {
            // If there is no previous superblock, then we start pointing
            // to the beginning of the block.
            this_superblock_start_index = 0;
        }

        // Problem: When in-block return 0 because i == 0, then +1
        // return the wrong last 1. Or does that even matter?
        //
        // I currently do not handle i % b == 0, right?
        if i % self.b as u64 == 0 {
            // The i-th 1 is the last 1 in the superblock.
            // So the superblock end is the i-th 1: return it.
            return Ok(previous_superblock_end_index);
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

        assert!(
            this_superblock_start_index <= this_superblock_end_index as u64,
            "this_superblock_start_index={} <= this_superblock_end_index={}",
            this_superblock_start_index,
            this_superblock_end_index
        );

        let in_block_offset: u64;

        let i_excluding_previous_superblocks: u64;

        if superblock_number == 0 {
            // No previous superblock, so all i ones/zeroes are to be found
            // in this block.
            i_excluding_previous_superblocks = i;
        } else {
            // Only count the previous superblocks.
            i_excluding_previous_superblocks = i - (superblock_number * self.b as u64);
        }

        println!(
            "{} superblock_number={} b={} i={} i-inside={}",
            space(self.is1, self.is_subblock),
            superblock_number,
            self.b,
            i,
            i_excluding_previous_superblocks
        );

        match self.in_superblock[superblock_number as usize] {
            InSuperblockSelect::Naive(ref naive) => {
                // What do I pass as i here? 0 == beginning of block.
                // But if it returns 0,
                println!(
                    "Naive select super_number={} b={} i={} i-inside={}",
                    superblock_number, self.b, i, i_excluding_previous_superblocks
                );

                in_block_offset = naive.select(i_excluding_previous_superblocks)?;

                println!("returned");
            }
            InSuperblockSelect::Subblock(ref subblock) => {
                // What data to pass here?
                println!(
                    "{} Subblock select super_number={} b={} i={} i-inside={}",
                    space(self.is1, self.is_subblock),
                    superblock_number,
                    self.b,
                    i,
                    i_excluding_previous_superblocks
                );

                in_block_offset = subblock.select(
                    &data[this_superblock_start_index as usize..=this_superblock_end_index],
                    i_excluding_previous_superblocks,
                )?;

                println!("returned");
            }
            InSuperblockSelect::LookupTable => {
                // What data to pass here?
                // Need block beginning to end.
                // And if its the last block, beginning of block to end of
                // global data.
                println!(
                    "{} Lookup table select super_number={} b={} i={} i-inside={} from {} to {}",
                    space(self.is1, self.is_subblock),
                    superblock_number,
                    self.b,
                    i,
                    i_excluding_previous_superblocks,
                    this_superblock_start_index,
                    this_superblock_end_index
                );
                in_block_offset = self.lookup_table_select(
                    &data[this_superblock_start_index as usize..=this_superblock_end_index],
                    i_excluding_previous_superblocks,
                );
                println!("returned");
            }
        }

        return Ok(this_superblock_start_index + in_block_offset);
    }

    fn in_superblock_for(
        data: &[bool],
        n: usize,
        is1: bool,
        superblock_start: usize,
        superblock_end: usize,
        is_subblock: bool,
    ) -> InSuperblockSelect {
        // + 1 here because end is included and otherwise not counted.
        let size = superblock_end + 1 - superblock_start;

        let result: InSuperblockSelect;

        assert!(
            superblock_start <= superblock_end,
            "{} n: {} this_superblock_start_index={} <= superblock_end={}",
            space(is1, is_subblock),
            n,
            superblock_start,
            superblock_end
        );

        if !is_subblock {
            // Naive or subblock.

            if size as f64 >= (n as f64).log2().powf(4.0) {
                // Naive.
                println!(
                    "{} block=naive: superblock_start: {} superblock_end: {} size: {} {:?}",
                    space(is1, is_subblock),
                    superblock_start,
                    superblock_end,
                    size,
                    &data[superblock_start..=superblock_end]
                );

                result = InSuperblockSelect::Naive(Select1Naive::new(
                    &data[superblock_start..=superblock_end],
                    is1,
                ));
            } else {
                // todo
                // Problem: It can happen that b == 0, because there is
                // only one element inside.
                //
                // Problem hereby is, that the size is supposed to be 7,
                // and that is already covered by the two previous naive blocks.
                //
                // So.. this 3rd block essentially repeats a block, which is
                // bad.
                // Ah not that is wrong as well. Its the last main superblock.

                // Subblock.
                println!(
                    "{} block=subblock: superblock_start: {} superblock_end: {} n: {} b: {} size: {} data: {:?}",
                    space(is1, is_subblock), superblock_start, superblock_end, n, (n as f32).log2().floor(), size, &data[superblock_start..=superblock_end]
                );

                result = InSuperblockSelect::Subblock(Select1::new(
                    &data[superblock_start..=superblock_end],
                    is1,
                    true,
                ));
            }
        } else {
            // Naive or lookup table.
            if size as f64 >= (n as f64).log2() {
                // Naive
                println!(
                    "{} block=naive: superblock_start: {} superblock_end: {}  size: {} data: {:?}",
                    space(is1, is_subblock),
                    superblock_start,
                    superblock_end,
                    //&data[superblock_start..=superblock_end],
                    size,
                    &data[superblock_start..=superblock_end],
                );

                result = InSuperblockSelect::Naive(Select1Naive::new(
                    &data[superblock_start..=superblock_end],
                    is1,
                ));
            } else {
                println!(
                    "{} block=lookup_table: superblock_start: {} superblock_end: {} size: {} data: {:?}",
                    space(is1, is_subblock),
                    superblock_start,
                    superblock_end,
                    size,
                    &data[superblock_start..=superblock_end]
                );
                result = InSuperblockSelect::LookupTable;
            }
        }

        return result;
    }

    fn lookup_table_select(&self, data: &[bool], i: u64) -> u64 {
        // Problem: block with 2 bits is passed in but
        // lookup_table only contains 3-bit blocks to look up.
        println!(
            "lookup_table_select: block={:?} i={} lookup_table: {:#?}",
            data, i, self.lookup_table
        );

        // Extend lookup-block if necessary.

        if data.len() == self.lookup_table_block_size as usize {
            return self.lookup_table[data][&i];
        } else {
            let block_size: usize = self.lookup_table_block_size as usize;
            let mut filled_block: Vec<bool> = Vec::with_capacity(block_size);
            filled_block.extend_from_slice(data);
            while filled_block.len() < block_size {
                filled_block.push(false);
            }

            return self.lookup_table[&filled_block][&i];
        }
    }

    pub fn select_simple(&self, data: &[bool], i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Ok(0);
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

pub fn space(is1: bool, is_subblock: bool) -> String {
    let is_one = if is1 { "1" } else { "0" };
    if is_subblock {
        return is_one.to_owned() + "    ";
    } else {
        return is_one.to_owned();
    }
}
