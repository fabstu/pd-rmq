use super::MyError;

use std::collections::HashMap;

// This is inside a block. So have to make index relative to this block.
#[derive(MallocSizeOf, Clone)]
pub struct Select1Naive {
    // size of bitvector
    n: u32,
    // Why do I have b here? Its the superblock #1s, and this here
    // is inside the block.
    //
    // Well, guesss it doesn't matter. Except maybe for performance..
    b: u32,
    // #1s in block. Why is this here?
    // Isn't this redundant with length of answers?
    //
    // And why do I compute this here?
    // Otherwise have to pass that in.
    //k: u32,

    // Might not be most efficient to store that way.
    //
    // Maybe use bare array? But doesnt that need size known beforehand?
    // Dunno enough about Rust.
    //
    // Anyway, might choose to use a different hashing algo that is faster.
    answers: HashMap<u32, u32>,
}

impl Select1Naive {
    pub fn new(data: &[bool], is1: bool) -> Self {
        let n = data.len();
        //let k = data.iter().filter(|v| **v == true).count();
        // Not sure whether floor or ceil.
        let b = (n as f64).log2().powf(2.0).floor() as u32;

        let mut answers: HashMap<u32, u32> = HashMap::new();

        let mut count = 0;
        for (i, &val) in data.iter().enumerate() {
            // Skip if not the value we are looking to count: 0 or 1
            // respectively.
            if val != is1 {
                continue;
            }

            // We are 1-based, right? So count+1 so the 1st 1 is at index 1.
            // But.. why do I keep the answers[0] entry around then?
            // And need an entry for size == b?
            //
            // So if I use array later,
            // then maybe better make this zero-based if using an array.
            // But since its a hashmap, I can keep it 1-based without MORE
            // OVERHEAD the hashmap creates anyway.
            //
            // Is select0(0) supposedly returning 0 an argument for making
            // this zero-based?
            // No, because I would store a non-zero value here, so I'd have
            // to special-case in some way.
            count += 1;
            answers.insert(count, i as u32);
        }

        Self {
            n: n as u32,
            b: b,
            //k: k as u32,
            answers: answers,
        }
    }

    pub fn select(&self, i: u64) -> Result<u64, MyError> {
        if i == 0 {
            return Err(MyError::Select1GotZero);
        }

        if i as u32 >= self.n {
            return Err(MyError::Select1OutOfBounds);
        }

        if i as u32 > self.answers.len() as u32 {
            return Err(MyError::Select1NotEnough1s);
        }

        return self
            .answers
            .get(&(i as u32))
            .ok_or(MyError::Select1OutOfBounds)
            .map(|v| *v as u64);
    }
}
