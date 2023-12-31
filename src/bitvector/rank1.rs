use std::collections::HashMap;

type TupleKey = (Vec<bool>, u64);

use super::u64_to_vec_bool;

use core::cmp::min;

use super::super::debug::DEBUG;

// Have Rank1 data here to be able to keep initializer here aswell.
#[derive(MallocSizeOf, Clone)]
pub struct Rank1 {
    block_size: u64,
    superblock_size: u64,
    // For each superblock, we store the number of 1s to the start of the
    // superblock.
    rank1_superblock_1s: Vec<u64>,
    // Number of 1s from start of superblock to the start of the block.
    rank1_block_1s: Vec<Vec<u64>>,
    // Lookup of 1s up to the given position: (block, offset_in_block)
    rank1_lookup_table: HashMap<TupleKey, u32>,
    lookup_table_block_size: u32,
}

#[allow(dead_code)]
impl Rank1 {
    pub fn new(data: &Vec<bool>) -> Self {
        let n = data.len() as f64;

        // Choose block_size much smaller than n.
        let block_size = (n.log2() / 2.0).floor().round() as u64;

        // Multiple of block_size.
        let superblock_size = block_size.pow(2) as u64;

        let superblock_count = (n / superblock_size as f64).floor() as u64;
        let block_count = (n / block_size as f64).floor() as u64;

        let block_count_per_superblock = block_count / superblock_count + 1;

        if DEBUG {
            println!(
            "rank1::new - block_size: {} superblock_size: {} block_count: {} superblock_count: {} block_count_per_superblock: {}",
            block_size, superblock_size, block_count, superblock_count, block_count_per_superblock
        );

            println!("rank1: Allocating superblock_1s");
        }

        //
        // Initialize block and superblock arrays.
        //
        // superblock_1s[superblock] -> #1s in superblock.
        //
        let mut superblock_1s = vec![0u64; (superblock_count + 1) as usize];

        if DEBUG {
            println!("rank1: Allocating block_1s");
        }

        // Takes too much space.
        // Problem: block_count is the number of ALL blocks.
        // But I only need the blocks for each superblock.

        // block_1s[superblock][block] -> #1s in block.
        let mut block_1s = vec![
            vec![0u64; (block_count_per_superblock + 1) as usize];
            (superblock_count + 1) as usize
        ];

        if DEBUG {
            println!("rank1: Initializing...");
        }

        //
        // Calc 1s up to each superblock.
        // Calc 1s from start of superblock to following blocks inside
        // superblock.
        //
        // But.. do I record blocksize and superblock_size at the start
        // or at the end of the block/superblock -> meaning do I record
        // for the block end end-size or the 1s up to the start?
        //
        // And.. what does this have to do with block/super being 1-indexed or
        // 0-indexed?
        //
        // Well, let's just say we record the start of the block by definition.
        //
        let mut rank = 0;
        let mut superblock_rank = 0;
        for (i, &bit) in data.iter().enumerate() {
            if i % superblock_size as usize == 0 {
                let superblock_index = i / superblock_size as usize;

                superblock_1s[superblock_index] = rank;
                superblock_rank = rank;

                if DEBUG {
                    println!(
                        "Updating superblock={} i: {} data.len: {} superblock_rank: {} bit: {}",
                        superblock_index,
                        i,
                        data.len(),
                        superblock_rank,
                        bit
                    );
                }
            }

            // No need to first scope to superblock_size because
            // block_size it is divisible by block_size.
            if i % block_size as usize == 0 {
                // Cuts off block-part by converting to usize.
                let superblock_index = i / superblock_size as usize;

                // Go into superblock, and then into block.
                let block_index = (i % superblock_size as usize) / block_size as usize;

                // Remove rank of superblock.
                let block_rank = rank - superblock_rank;

                // How can I get a too-big block_index here?
                // 17 when max passable index is block_size=16
                //
                // Maybe because we round down when calculating block_bitsize?
                // This would mean the last block is not covered, as is the case
                // in select1.

                if DEBUG {
                    println!(
                        "Updating superblock={} block={}: i: {} data.len: {}block_rank: {} bit: {}",
                        superblock_index,
                        block_index,
                        i,
                        data.len(),
                        block_rank,
                        bit
                    );
                }

                block_1s[superblock_index][block_index] = block_rank;
            }

            // rank1 does not include the current position, so only count up
            // after.
            if bit {
                rank += 1;
            }
        }

        // Build rank1 lookup table.
        //
        // But.. maybe using array indexes for lookup is faster though.
        let mut rank1_lookup_table: HashMap<TupleKey, u32> = HashMap::new();

        let last_value = 2u64.pow(block_size as u32);

        // Problem: lookup table is too big.
        //
        // It currently needs for block_size=32 2^32 *32 iterations
        // to create the 32-bit table.

        // All possible values for block_size.
        for i in 0..2u64.pow((block_size) as u32) {
            if DEBUG {
                println!(
                    "Creating lookup table block_size={} i={} last_value={}",
                    block_size, i, last_value
                );
            }

            // Get block bitvector pattern.
            let block = u64_to_vec_bool(i, block_size);

            for lookup in 0..block_size {
                let key = (block.clone(), lookup);

                // How do I get #1s in i up to lookup without calcing rank1 for
                // each lookup manually?
                //
                // a) flip each bit (block_bitsize is bit-count) manually, and
                //    update #1s on each flip.

                let rank1: u32;

                if lookup == 0 {
                    rank1 = 0;
                } else {
                    // Create bitmask containing bits up to lookup.
                    let mask = (1u64 << (lookup)) - 1;

                    // Get bits up to lookup.
                    let bits_up_to_lookup = i & mask;

                    // Calc rank1 using count_ones built-in.
                    rank1 = bits_up_to_lookup.count_ones();
                }

                // println!(
                //     "ins block_bitsize: {} i: {} block: {:#?} lookup: {} rank1: {}",
                //     block_bitsize, i, block, lookup, rank1
                // );

                rank1_lookup_table.insert(key, rank1);
            }
        }

        // println!("block_bitsize: {} superblock_bitsize: {}, block_size: {}, superblock_size: {}, rank1_sb_1s: {}, rank1_block_1s: {}, lookup_count: {}", block_bitsize, superblock_bitsize, block_size, superblock_size, superblock_1s.len(), block_1s.len(), rank1_lookup_table.len());

        Self {
            block_size: block_size,
            superblock_size: superblock_size,
            rank1_superblock_1s: superblock_1s,
            rank1_block_1s: block_1s,
            rank1_lookup_table: rank1_lookup_table,
            lookup_table_block_size: block_size as u32,
        }
    }
}

impl Rank1 {
    pub fn rank1(&self, data: &[bool], i: u64) -> u64 {
        //let superblock_index = i / self.superblock_size;

        // Cuts off block-part by converting to usize.
        let superblock_index = (i / self.superblock_size) as usize;

        // Go into superblock, and then into block.
        let block_index = ((i % self.superblock_size) / self.block_size) as usize;

        let superblock_offset = superblock_index * self.superblock_size as usize;

        let block_start = superblock_offset + block_index * self.block_size as usize;

        // Except for end of slize.
        //
        // But.. aren't we supposed to only count to the block?
        // Yes. Here we just prepare the block for lookup later.
        let mut block_end = block_start + self.block_size as usize;

        // Account for the last block not being completely filled.
        block_end = min(block_end, data.len());

        if DEBUG {
            println!(
            "rank1: i: {} superblock_index: {} block_index: {} block_start: {} block_end: {} superblock_size: {} block_size: {}",
            i, superblock_index, block_index, block_start, block_end, self.superblock_size, self.block_size
        );
        }

        // Copying might be slow, but current alternative is conversion to
        // u32/64.
        //
        // Anyway, only have to copy here because I have to reverse.
        // Think about why that is later... .
        //
        // a) Isn't block stored from smallest to biggest?
        //    Why do I get the correct block with block_start and block_end,
        //    but reversed compared to lookup table?
        //    Is lookup table created biggest to smallest?
        //
        //    Seems conversion fron u64 to Vec<bool> is done from biggest to
        //    smallest. When fixing it: Have to adapt TupleKey because
        //    [TupleKey] does not allow substitution using &[bool] like
        //    recursive HashMap lookup_table[&block[..]]lookup] does.
        let block = &data[block_start..block_end];

        // Get index inside block. Don't have to divide by superblock_size first
        // because that is dvidable by block_size by definition.
        let lookup = i % self.block_size;

        if DEBUG {
            println!(
                "Superblock rank1: {} block-rank1: {} lookup-rank1: {}",
                self.rank1_superblock_1s[superblock_index],
                // Does for block_index == 0 this store the rank1 up to the 1st
                // block, or rather after the 1st block?
                self.rank1_block_1s[superblock_index][block_index],
                self.lookup_table_rank1(block, lookup)
            );
        }

        // Problem: With i=151 and superblock-size being 151, this somehow
        // still includes the block with block_start from 151 to 164.

        return self.rank1_superblock_1s[superblock_index]
            + self.rank1_block_1s[superblock_index][block_index]
            + self.lookup_table_rank1(block, lookup);
    }

    fn lookup_table_rank1(&self, block: &[bool], lookup: u64) -> u64 {
        let result: u64;

        if block.len() == self.lookup_table_block_size as usize {
            result = self.rank1_lookup_table[&(block.to_vec(), lookup)] as u64;
        } else {
            let block_size: usize = self.lookup_table_block_size as usize;
            let mut filled_block: Vec<bool> = Vec::with_capacity(block_size);
            filled_block.extend_from_slice(block);
            while filled_block.len() < block_size {
                filled_block.push(false);
            }

            result = self.rank1_lookup_table[&(filled_block, lookup)] as u64;
        }

        if DEBUG {
            println!(
                "Lookup table rank1: block_size: {} block: {:?} lookup: {} -> {}",
                self.lookup_table_block_size, block, lookup, result
            );
        }

        return result;
    }

    pub fn rank0(&self, data: &[bool], i: u64) -> u64 {
        return i - self.rank1(data, i);
    }

    pub fn rank1_simple(&self, data: &[bool], i: u64) -> u64 {
        let mut count = 0;
        for j in 0..i {
            if data[j as usize] {
                count += 1;
            }
        }
        count
    }
}
