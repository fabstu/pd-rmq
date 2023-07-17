use std::collections::HashMap;

use super::u64_to_vec_bool;
use std::cmp::max;

#[derive(MallocSizeOf, Clone, Debug)]
pub struct SelectLookupTable {
    lookup_table: HashMap<Vec<bool>, HashMap<u64, u64>>,
    max_lookup_bits: u32,

    is1: bool,
}

impl SelectLookupTable {
    pub fn new(is1: bool) -> Self {
        Self {
            max_lookup_bits: 0,
            lookup_table: HashMap::new(),
            is1: is1,
        }
    }

    pub fn encountered(&mut self, lookup_bits: u32) {
        self.max_lookup_bits = max(self.max_lookup_bits, lookup_bits);
    }

    pub fn create(&mut self) {
        let maximum_block_size_in_bits = self.max_lookup_bits;

        let mut lookup_table: HashMap<Vec<bool>, HashMap<u64, u64>> = HashMap::new();

        // All possible values for block_size.
        //
        // This might be too much.. .
        for i in 0..2u64.pow((maximum_block_size_in_bits) as u32) {
            if i % 100 == 0 {
                println!(
                    "Initializing lookup table for i={} from 0 to 2^{} = {}",
                    i,
                    maximum_block_size_in_bits,
                    2u64.pow(maximum_block_size_in_bits as u32)
                );
            }

            // Get block bitvector pattern.
            //
            // What is the bit-size of the blocks?
            // One case I had was where the input-blockrange was 2bit
            // but the lookup_table blocksize was 3bit, making the app crash.
            //
            // So.. -1 for now.
            let block = u64_to_vec_bool(i, maximum_block_size_in_bits as u64);
            let mut block_lookups: HashMap<u64, u64> = HashMap::new();

            let number_of_ones_zeroes = block.iter().filter(|v| **v == self.is1).count() as u64;

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
                        if block[index] != self.is1 {
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

            lookup_table.insert(block, block_lookups);
        }

        // How do I know the maximum #bits I need for the lookup table?
        // a) b: #bits per superblock.
        // b) size < log^4 n: Only using lookup table in this case.
        // c) Observe the maximum block size from previous iteration.

        self.lookup_table = lookup_table;
    }

    pub fn lookup(&self, data: &[bool], i: u64) -> u64 {
        // Problem: block with 2 bits is passed in but
        // lookup_table only contains 3-bit blocks to look up.
        println!(
            "lookup_table_select: block={:?} i={} lookup_table: {:#?}",
            data, i, self.lookup_table
        );

        if data.len() == self.max_lookup_bits as usize {
            // Block has correct length.
            return self.lookup_table[data][&i];
        } else {
            // Extend lookup-block if necessary.
            let block_size: usize = self.max_lookup_bits as usize;
            let mut filled_block: Vec<bool> = Vec::with_capacity(block_size);
            filled_block.extend_from_slice(data);
            while filled_block.len() < block_size {
                filled_block.push(false);
            }

            return self.lookup_table[&filled_block][&i];
        }
    }
}
