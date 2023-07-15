use super::Bitvector;

impl Bitvector {
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
}
