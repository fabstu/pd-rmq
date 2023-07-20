use crate::debug::DEBUG;

use super::RMQError;
use super::RMQ;
use std::collections::HashMap;

use super::rmq_sparse::RMQSparse;

#[derive(MallocSizeOf, Clone)]
pub struct RMQSpanningBlocks {
    block_size: usize,
    block_count: usize,
    // Necessary for lookup of positions.
    //numbers: Vec<u64>,
    //2:
    //block_minimum: Vec<u64>,
    // Allows range minimum query over whole blocks.
    block_minimum_sparse: RMQSparse,
    // Wanna return position of minimum, not minimum itself.
    block_minimum_position_in_block: Vec<usize>,

    //1:3:
    cartesian_trees: CartesianTrees,
}

impl RMQSpanningBlocks {
    pub fn new(numbers: Vec<u64>) -> Self {
        let n = (numbers.len() - 1) as f64;

        let block_size = (n.log2() / 4.0).ceil() as usize;
        let block_count = (n / block_size as f64).ceil() as usize;

        // Query types:
        // 1) Zwei Teilblöcke + mehrere Blöcke
        // 2) umfasst ganze Blöcke: Blockgrenze zu Blockgrenze
        // 3) 1-2 Teilblöcke: Innerhalb eines Blocks oder eine grenze kreuzend.

        //2:
        // Stores minimum per whole block.
        let mut block_minimum = vec![0u64; block_count as usize];

        let block_minimum_position_in_block = vec![0usize; block_count as usize];

        // Fill block_minimum and block_minimum_position_in_block.
        let mut min = std::u64::MAX;
        for i in 0..numbers.len() {
            // Is i included in this block_minimum or only in the next?
            //
            // Determines whether to do this before or after.
            if i % block_size == 0 {
                // Store minimum of previous block.
                block_minimum[i / block_size] = min;
                min = std::u64::MAX;
            }

            min = std::cmp::min(min, numbers[i]);
        }

        // Write last block.
        if numbers.len() % block_size == 0 {
            // Store minimum of last block.
            block_minimum[block_count - 1] = min;
        }

        // Verwende n log n-DS Sparse Table für B.
        let block_minimum_sparse = RMQSparse::new(block_minimum);

        let cartesian_trees = CartesianTrees::new(&numbers, block_size, block_count);

        return Self {
            block_size: block_size,
            block_count: block_count,
            //numbers: numbers,
            //block_minimum: block_minimum,
            block_minimum_sparse: block_minimum_sparse,
            block_minimum_position_in_block: block_minimum_position_in_block,
            cartesian_trees: cartesian_trees,
        };
    }

    fn number(&self, index: usize) -> u64 {
        return self.block_minimum_sparse.numbers[index];
    }

    pub fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        let mut from_block = from / self.block_size;
        let from_block_offset = from % self.block_size;
        let mut to_block = to / self.block_size;
        let to_block_offset = to % self.block_size;

        if DEBUG {
            println!(
                "from_block: {}, from_block_offset: {}, to_block: {}, to_block_offset: {}",
                from_block, from_block_offset, to_block, to_block_offset
            );
        }

        if to_block - from_block != 0 {}

        // Same element.
        if from_block_offset == to_block_offset && from_block == to_block {
            return Ok(from);
        }

        // 2: Whole blocks.
        if from_block_offset == 0 && to_block_offset == 0 {
            // Whole blocks.
            if DEBUG {
                println!(
                "case2: from_block: {}, to_block: {}, from_block_offset: {}, to_block_offset: {}",
                from_block, to_block, from_block_offset, to_block_offset
            );
            }

            let block_minimum = self
                .block_minimum_sparse
                .range_minimum_query(from_block, to_block)?;

            return Ok(block_minimum);
        }

        // 3: Inside same block.
        //
        // If either offset is non-zero, do this case.
        if from_block_offset != 0 || to_block_offset != 0 && from_block == to_block {
            if DEBUG {
                println!(
                "case3.1: from_block: {}, to_block: {}, from_block_offset: {}, to_block_offset: {}",
                from_block, to_block, from_block_offset, to_block_offset
            );
            }

            return Ok(self.cartesian_trees.range_minimum_query(
                from_block,
                from_block_offset,
                to_block_offset,
            ));
        }

        // 3: No blocks in-between with going over block border.
        if from_block_offset != 0 && to_block_offset != 0 && from_block + 1 == to_block {
            if DEBUG {
                println!(
                "case3.2: from_block: {}, to_block: {}, from_block_offset: {}, to_block_offset: {}",
                from_block, to_block, from_block_offset, to_block_offset
            );
            }

            let from_minimum_index = self.cartesian_trees.range_minimum_query(
                from_block,
                from_block_offset,
                // Not sure whether to is correct here.
                //
                // Whether it is supposed to include the last element
                // or not.
                self.block_size - 1,
            );
            let from_min = self.number(from_minimum_index);

            let to_minimum_index =
                self.cartesian_trees
                    .range_minimum_query(to_block, 0, to_block_offset);
            let to_minimum = self.number(to_minimum_index);

            if from_min < to_minimum {
                return Ok(from_minimum_index);
            } else {
                return Ok(to_minimum_index);
            }
        }

        // 1: Blocks and one or two partial blocks.
        if DEBUG {
            println!(
                "case1: from_block: {}, to_block: {}, from_block_offset: {}, to_block_offset: {}",
                from_block, to_block, from_block_offset, to_block_offset
            );
        }

        let mut min = std::u64::MAX;
        let mut min_index = 0;

        if from_block_offset != 0 {
            // Do not include this block.
            let from_minimum_index = self.cartesian_trees.range_minimum_query(
                from_block,
                from_block_offset,
                self.block_size - 1,
            );

            let from_minimum = self.number(from_minimum_index);

            min = from_minimum;
            min_index = from_minimum_index;

            from_block += 1;

            if DEBUG {
                println!(
                    "case1.result: from_minimum_index: {} from_minimum: {}",
                    from_minimum_index, from_minimum
                );
            }
        }

        if to_block_offset != 0 {
            let to_minimum_index =
                self.cartesian_trees
                    .range_minimum_query(to_block, 0, to_block_offset);

            let to_minimum = self.number(to_minimum_index);

            if to_minimum < min {
                min = to_minimum;
                min_index = to_minimum_index;
            }

            if DEBUG {
                println!(
                    "case1.result: to_minimum_index: {} to_minimum: {}",
                    to_minimum_index, to_minimum
                );
            }

            // Do not include this block.
            //
            // Except.. is block_minimum_sparse inclusive or exclusive for to?
            to_block -= 1;
        }

        let block_minimum_index = self
            .block_minimum_sparse
            .range_minimum_query(from_block, to_block)?;

        let block_minimum = self.number(block_minimum_index);

        if block_minimum < min {
            min = block_minimum;
            min_index = block_minimum_index;
        }

        if DEBUG {
            println!(
                "case1.result: block_minimum_index: {} block_minimum: {}",
                block_minimum_index, block_minimum
            );

            print!("case1.result: end: min_index: {}, min: {}", min_index, min);
        }

        return Ok(min_index);
    }
}

impl RMQ for RMQSpanningBlocks {
    fn new(numbers: Vec<u64>) -> Self {
        RMQSpanningBlocks::new(numbers)
    }

    fn range_minimum_query(&self, from: usize, to: usize) -> Result<usize, RMQError> {
        RMQSpanningBlocks::range_minimum_query(self, from, to)
    }
}

#[derive(MallocSizeOf, Clone)]
struct CartesianTrees {
    s: usize,
    // Precomputed lookup table:
    //   cartesian_tree_number -> [from][to]
    //
    // Not sure whether cartesian_tree_number needs 32 or 64 bit.
    // s = log n / 2
    // We get n = 1 000 000 -> s aufgerundet 5
    //
    // Should be enough either anyway.
    cartesian_trees: HashMap<u64, Vec<Vec<usize>>>,
    cartesian_tree_number_for_blocks: Vec<u64>,
}

impl CartesianTrees {
    pub fn new(array: &Vec<u64>, block_size: usize, block_count: usize) -> Self {
        let n_float = array.len() as f64;

        // Number of nodes in cartesian tree.
        let s = (n_float.log2() / 4.0).ceil() as usize;

        let mut cartesian_trees: HashMap<u64, Vec<Vec<usize>>> = HashMap::new();
        let mut cartesian_tree_number_for_blocks: Vec<u64> = vec![0u64; block_count];

        for i in 0..block_count {
            let block_start = i * block_size;
            let block_end = (i + 1) * block_size;

            // Calculate cartesian tree number for block.
            let cartesian_tree_number = Self::cartesian_tree_number(&array[block_start..block_end]);

            cartesian_tree_number_for_blocks[i] = cartesian_tree_number;

            if cartesian_trees.contains_key(&cartesian_tree_number) {
                continue;
            }

            if DEBUG {
                println!(
                    "Precomputing cartesian tree number i={} cart_tree_number={:#b} n: {} s: {} block_size: {}, block_count: {}",
                    i, cartesian_tree_number,
                    n_float as u32,
                    s,
                    block_size, block_count
                );
            }

            // Precompute RMQ structure for cartesian tree number.
            cartesian_trees.insert(
                cartesian_tree_number,
                Self::precompute_for_cartesian_tree(&array[block_start..block_end]),
            );
        }

        // Calculate cartesian tree number for each block.
        // If cartesian tree number is already calculated, don't recompute,
        // else precompute RMQ structure.

        // Representable using 2s + 1 bits -> succinct trees.
        // -> jeder mögliche cartesian tree mit 2^(2s + 1) bits darstellbar.
        //
        // -> Speichere für jeden möglichen cartesian tree und jede mögliche
        // start-und Endposition das Ergebnis des rmq.
        //
        // !q Was ist der Unterschied zum direkt abspeichern aller Lösungen?
        //
        // Nur Wurzel(n) viele kartesische Bäume.
        // Nur log^2(n) viele Start- und Endpositionen.
        // -> Nur linear viel Platz.
        //
        // Lookup table: Lösung für Teilstück des blocks.

        Self {
            s: s,
            cartesian_trees: cartesian_trees,
            cartesian_tree_number_for_blocks: cartesian_tree_number_for_blocks,
        }
    }

    fn range_minimum_query(&self, block_number: usize, from: usize, to: usize) -> usize {
        if DEBUG {
            println!(
                "CartesianTrees::range_minimum_query block_number: {}, from: {}, to: {}",
                block_number, from, to
            );
        }

        return self.cartesian_trees[&self.cartesian_tree_number_for_blocks[block_number]][from]
            [to];
    }

    /// Calculates cartesian tree number for given block.
    ///
    /// Uses push and pop operations that construction would entail for this.
    fn cartesian_tree_number(block: &[u64]) -> u64 {
        assert_ne!(block.len(), 0);
        assert!(
            // For each number, one bit for push and one for pop.
            block.len() < 32,
            "block too large for cartesian tree number"
        );

        let mut stack: Vec<u64> = Vec::new();

        let mut cartesian_number = 0;

        for number in block {
            // Pop all elements from stack that are greater than number.
            while stack.len() > 0 && stack[stack.len() - 1] > *number {
                stack.pop();

                // Keep new bit zero for pop.
                cartesian_number <<= 1;
            }

            // Push number onto stack.
            stack.push(*number);

            // Set new bit to 1 for push.
            cartesian_number <<= 1;
            cartesian_number |= 1;
        }

        // Pop stack.
        cartesian_number <<= stack.len();

        return cartesian_number;
    }

    fn precompute_for_cartesian_tree(block: &[u64]) -> Vec<Vec<usize>> {
        let s = block.len();

        let mut cartesian_tree_rmq: Vec<Vec<usize>> = vec![vec![0usize; s]; s];

        // Precompute all possible [From][To] combinations.
        //
        // a) [x] for i in 0..block.len(); for j in i..block.len()
        //        But.. then what did I compute the cartesian tree for?
        //        Can do that without anyway.
        // b)     Use cartesian tree more than just a lookup of min value
        //        -> walk cartesian tree?
        //        - I know min from left to right.
        for i in 0..block.len() {
            // Minimum to itself.
            cartesian_tree_rmq[i][i] = i;

            for j in i + 1..block.len() {
                if DEBUG {
                    println!("Precomputing i: {}, j: {}", i, j);
                }

                // We only grow, so with each step we take on a new number.
                //
                // If the new number is smaller than the current, then record
                // it. Else keep the current minimum.
                if block[cartesian_tree_rmq[i][j - 1]] < block[j] {
                    cartesian_tree_rmq[i][j] = cartesian_tree_rmq[i][j - 1];
                } else {
                    cartesian_tree_rmq[i][j] = j;
                }
            }
        }

        return cartesian_tree_rmq;
    }

    // fn set_right_child(node: u64, new_right_child: u32) -> u64 {
    //     let (value, left_child, _) = Self::unpack_values(node);

    //     return Self::pack_values(value, left_child, new_right_child);
    // }

    // fn pack_values(value1: u32, value2: u32, value3: u32) -> u64 {
    //     assert!(value1 < (1 << 20));
    //     assert!(value2 < (1 << 20));
    //     assert!(value3 < (1 << 20));

    //     // Shift left and or-ing to pack.
    //     let packed = ((value1 as u64) << 40) | ((value2 as u64) << 20) | (value3 as u64);
    //     packed
    // }

    // fn unpack_values(packed: u64) -> (u32, u32, u32) {
    //     let value1 = ((packed >> 40) & ((1 << 20) - 1)) as u32;
    //     let value2 = ((packed >> 20) & ((1 << 20) - 1)) as u32;
    //     let value3 = (packed & ((1 << 20) - 1)) as u32;
    //     (value1, value2, value3)
    // }
}

mod tests {

    #[allow(unused_imports)]
    use super::CartesianTrees;

    #[test]
    fn test_cartesian_tree_number() {
        let array = vec![3, 2, 4, 4, 5, 2, 4, 7, 6];

        let cartesian_tree_number = CartesianTrees::cartesian_tree_number(&array);

        assert_eq!(cartesian_tree_number, 0b101111000111010000);
    }
}
