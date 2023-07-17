
# Implementations

a) Bitvector with rank0/1 and select0/1 proposed in lecture.
b) Adapted predecessor Elias-Fano proposed in lecture.
c) RMQ Naive fast space=O(n^2) and slow space=O(n) -> cargo run pd
d) RMQ sparse -> cargo run rmq


## Finding issue with bitvector:

---- bitvector::testing_select1_thorough stdout ----
1 n=20, k=9, b=4
1 block=subblock: superblock_start: 0 superblock_end: 11 n: 20 b: 4 size: 12 data: [false, true, false, true, false, false, false, false, true, false, false, true]
1     n=12, k=4, b=3
1     block=naive: superblock_start: 0 superblock_end: 8  size: 9 data: [false, true, false, true, false, false, false, false, true]
1     Last: 
1     block=lookup_table: superblock_start: 9 superblock_end: 11 size: 3 data: [false, false, true]
1     Added superblock_end_indexes b: 3 superblock_end_indexes: [8] 
1 block=subblock: superblock_start: 12 superblock_end: 18 n: 20 b: 4 size: 7 data: [false, true, false, true, true, false, true]
1     n=7, k=4, b=2
1     block=naive: superblock_start: 0 superblock_end: 3  size: 4 data: [false, true, false, true]
1     block=naive: superblock_start: 4 superblock_end: 6  size: 3 data: [true, false, true]
1     Added superblock_end_indexes b: 2 superblock_end_indexes: [3, 6] 
1 Last: 
1 block=subblock: superblock_start: 19 superblock_end: 19 n: 20 b: 4 size: 1 data: [true]
1     n=1, k=1, b=0
thread 'bitvector::testing_select1_thorough' panicked at 'assertion failed: `(left != right)`
  left: `0`,
 right: `0`: b must not be 0. n=1', src/bitvector/select1/mod.rs:108:9

Observation:
- b = 4
- 9 trues, so 3 blocks are necessary anyway.
- 1st block ends with 4th true (correct)
- 2nd-block ends with true (correct)

Result:
- Must allow one-element blocks.
- No need to check for end-of-block, since n=1 means only one element fits.
- Adding in_superblock manually since Last:
- a) Set b = 1 when n = 1.
But:
- Can do using lookup table instead.

Question anyway:
- What about multiple lookups needing different amount of block-size? Probably have to generate for multiple block-sizes instead of the current monstrocity.

## Problem lookup_table not matching length.

- a) Convert all to u32
- b) Fill up with falses up to the lookup_table width.
  - Might have to copy.
- c) Special-case 


## Problem n=1 produces:

Select1: 9
1 superblock_number=2 b=4 i=9 i-inside=1
1 Subblock select super_number=2 b=4 i=9 i-inside=1
1     Accesing i=1 in data of len=1
thread 'bitvector::testing_select1_thorough' panicked at 'assertion failed: `(left == right)`
  left: `Ok(19)`,
 right: `Err(Select1OutOfBounds)`', src/bitvector/mod.rs:253:9

Observations:
- it wants to access 