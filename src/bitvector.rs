use std::collections::HashMap;
use std::error::Error;
use std::fmt;

// type BitVector = Vec<bool>;
type TupleKey = (Vec<bool>, u64);

#[derive(MallocSizeOf, Clone)]
pub struct Bitvector {
    block_bits: u64,
    superblock_bits: u64,
    block_size: u64,
    superblock_size: u64,
    // For each superblock, we store the number of 1s to the start of the
    // superblock.
    rank1_superblock_1s: Vec<u64>,
    // Number of 1s from start of superblock to the start of the block.
    rank1_block_1s: Vec<Vec<u64>>,
    // Lookup of 1s up to the given position: (block, offset_in_block)
    rank1_lookup_table: HashMap<TupleKey, u32>,

    data: Vec<bool>,
}

#[derive(Debug, PartialEq)]
pub enum MyError {
    InvalidValue,
    Select1GotZero,
    Select1NotEnough1s,
    Select1OutOfBounds,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MyError::InvalidValue => f.write_str("Invalid value in vector"),
            MyError::Select1GotZero => f.write_str("select1 got 0"),
            MyError::Select1NotEnough1s => f.write_str("select1 not enough 1s"),
            MyError::Select1OutOfBounds => {
                f.write_str("select1 out of bounds due to not enough 1s")
            }
        }
    }
}

impl Error for MyError {}

impl Bitvector {
    // Passes in a vector of 0s and 1s with lowest bits first.
    fn from_vec(vec: Vec<u8>) -> Result<Self, MyError> {
        if vec.iter().all(|b| *b == 1 || *b == 0) {
            Ok(Self::new(vec.iter().map(|b| *b == 1).collect()))
        } else {
            Err(MyError::InvalidValue)
        }
    }

    // Passes in a vector of 0s and 1s with lowest bits first.
    pub fn new(data: Vec<bool>) -> Self {
        let n = data.len() as f64;

        let block_bitsize = (n.log2() / 2.0).floor().round() as u64;
        let superblock_bitsize = block_bitsize * block_bitsize;

        let block_size = 2u64.pow(block_bitsize as u32);
        let superblock_size = 2u64.pow(superblock_bitsize as u32);

        //
        // Initialize block and superblock arrays.
        //
        // superblock_1s[superblock] -> #1s in superblock.
        //
        let mut superblock_1s = vec![0u64; (superblock_size + 1) as usize];
        //
        // block_1s[superblock][block] -> #1s in block.
        //
        let mut block_1s =
            vec![vec![0u64; (block_size + 1) as usize]; (superblock_size + 1) as usize];

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
                let superblock_index = superblock_size as usize;

                superblock_1s[superblock_index] = rank;
                superblock_rank = rank;
            }

            if i % block_size as usize == 0 {
                // Cuts off block-part by converting to usize.
                let superblock_index = i / superblock_size as usize;

                // Go into superblock, and then into block.
                let block_index = (i % superblock_size as usize) / block_size as usize;

                // Remove rank of superblock.
                let block_rank = rank - superblock_rank;

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

        // All possible values for block_size.
        for i in 0..2u64.pow((block_size) as u32) {
            for lookup in 0..block_size {
                let block = Self::u64_to_vec_bool(i, block_size);
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

                println!(
                    "ins block_bitsize: {} i: {} block: {:#?} lookup: {} rank1: {}",
                    block_bitsize, i, block, lookup, rank1
                );

                rank1_lookup_table.insert(key, rank1);
            }
        }

        println!("block_bitsize: {} superblock_bitsize: {}, block_size: {}, superblock_size: {}, rank1_sb_1s: {}, rank1_block_1s: {}, lookup_count: {}", block_bitsize, superblock_bitsize, block_size, superblock_size, superblock_1s.len(), block_1s.len(), rank1_lookup_table.len());

        Self {
            block_bits: block_bitsize,
            superblock_bits: superblock_bitsize,
            block_size: block_size,
            superblock_size: superblock_size,
            rank1_superblock_1s: superblock_1s,
            rank1_block_1s: block_1s,
            rank1_lookup_table: rank1_lookup_table,
            data: data,
        }
    }

    pub fn rank1(&self, i: u64) -> u64 {
        //let superblock_index = i / self.superblock_size;

        // Cuts off block-part by converting to usize.
        let superblock_index = (i / self.superblock_size) as usize;

        // Go into superblock, and then into block.
        let block_index = ((i % self.superblock_size) / self.block_size) as usize;

        let superblock_offset = superblock_index * self.superblock_size as usize;

        let block_start = superblock_offset + block_index * self.block_size as usize;

        let block_end = block_start + self.block_size as usize;

        let mut block = self.data[block_start..block_end].to_vec();

        // Reversing necessary because otherwise wrong order.
        block.reverse();

        let lookup = i % self.block_size;

        println!("block: {:?} lookup: {}", block, lookup,);

        println!(
            "block_loockedup: {}",
            self.rank1_lookup_table[&(block.clone(), lookup)] as u64
        );

        return self.rank1_superblock_1s[superblock_index]
            + self.rank1_block_1s[superblock_index][block_index]
            + self.rank1_lookup_table[&(block, lookup)] as u64;
    }

    pub fn rank0(&self, i: u64) -> u64 {
        return i - self.rank1(i);
    }

    // pub fn rank1_simple(&self, i: u64) -> u64 {
    //     let mut count = 0;
    //     for j in 0..i {
    //         if self.data[j as usize] {
    //             count += 1;
    //         }
    //     }
    //     count
    // }

    pub fn select1(&self, i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= self.data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return Ok(j);
            }
        }

        return Err(MyError::Select1NotEnough1s);
    }

    pub fn select1_simple(&self, i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }
        if i >= self.data.len() as u64 {
            return Err(MyError::Select1OutOfBounds);
        }

        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return Ok(j);
            }
        }

        return Err(MyError::Select1NotEnough1s);
    }

    pub fn select0(&self, i: u64) -> u64 {
        let mut count = 0;
        for j in 0..self.data.len() as u64 {
            if self.data[j as usize] {
                count += 1;
            }
            if count == i {
                return count;
            }
        }
        panic!("select out of bounds");
    }

    fn u64_to_vec_bool(n: u64, bit_size: u64) -> Vec<bool> {
        // Find out how many bits are required to represent the number.

        // Create the vector with each bit encoded as a bool.
        let mut vec = Vec::with_capacity(bit_size as usize);
        for i in 0..bit_size {
            let bit = (n >> (bit_size - 1 - i)) & 1;
            vec.push(bit == 1);
        }

        vec
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn testing_rank1() {
        let vec: Vec<u8> = vec![1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0];

        println!("vec: {:?}", vec);

        let bit_vector = Bitvector::from_vec(vec).unwrap();

        assert_eq!(bit_vector.rank1(0), 0);
        assert_eq!(bit_vector.rank1(1), 1);
        assert_eq!(bit_vector.rank1(2), 1);
        assert_eq!(bit_vector.rank1(3), 2);
        assert_eq!(bit_vector.rank1(4), 2);
        assert_eq!(bit_vector.rank1(5), 3);
        assert_eq!(bit_vector.rank1(6), 3);
        assert_eq!(bit_vector.rank1(7), 3);
        assert_eq!(bit_vector.rank1(8), 4);
        assert_eq!(bit_vector.rank1(9), 5);
        assert_eq!(bit_vector.rank1(10), 5);
        assert_eq!(bit_vector.rank1(11), 5);
        assert_eq!(bit_vector.rank1(12), 5);
        assert_eq!(bit_vector.rank1(13), 5);
        assert_eq!(bit_vector.rank1(14), 6);
        assert_eq!(bit_vector.rank1(15), 7);
    }

    #[test]
    fn testing_select1() {
        let vec: Vec<u8> = vec![1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0];

        println!("vec: {:?}", vec);

        let bit_vector = Bitvector::from_vec(vec).unwrap();

        let select1_fn = |i| bit_vector.select1(i);
        let select1_simple_fn = |i| bit_vector.select1_simple(i);

        testing_select1_variants(select1_fn);
        testing_select1_variants(select1_simple_fn);
    }

    fn testing_select1_variants<F>(select1: F)
    where
        F: Fn(u64) -> Result<u64, MyError>,
    {
        assert_eq!(select1(0).unwrap_err(), MyError::Select1GotZero);
        assert_eq!(select1(1).unwrap(), 0);
        assert_eq!(select1(2).unwrap(), 2);
        assert_eq!(select1(3).unwrap(), 4);
        assert_eq!(select1(4).unwrap(), 7);
        assert_eq!(select1(5).unwrap(), 8);
        assert_eq!(select1(6).unwrap(), 13);
        assert_eq!(select1(7).unwrap(), 14);
        // Only 7 1s in the bitvector.
        assert_eq!(select1(8).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(9).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(10).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(11).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(12).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(13).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(14).unwrap_err(), MyError::Select1NotEnough1s);
        assert_eq!(select1(15).unwrap_err(), MyError::Select1NotEnough1s);
        // Out of bounds of the bitvector, can never be that many 1s.
        assert_eq!(select1(16).unwrap_err(), MyError::Select1OutOfBounds);
    }
}
