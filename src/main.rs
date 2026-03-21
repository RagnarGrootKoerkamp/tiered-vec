use std::mem::replace;

fn main() {
    println!("Hello, world!");
}

/// 2 level structure:
/// A blocks of size B
pub struct TieredVec2<T, const A: usize, const B: usize> {
    /// Current number of elements
    n: usize,

    /// Pointer to the first element of each block.
    head: [usize; A],
    /// The actual blocks of data.
    blocks: [[T; B]; A],
}

// Notation:
// idx: external array index
// i: block index
// j: index within block in raw data

impl<T: Default, const A: usize, const B: usize> TieredVec2<T, A, B> {
    pub fn new() -> Self {
        Self {
            n: 0,
            head: [0; A],
            blocks: std::array::from_fn(|_| std::array::from_fn(|_| T::default())),
        }
    }

    fn index(&self, idx: usize) -> (usize, usize) {
        let i = idx / B;
        (i, (self.head[i] + idx % B) % B)
    }

    pub fn get(&self, idx: usize) -> &T {
        let (i, j) = self.index(idx);
        &self.blocks[i][j]
    }

    pub fn insert(&mut self, i: usize, value: T) {
        assert!(self.n < A * B, "TieredVec is already full");
        self.n += 1;
        let (i, j) = self.index(i);
        let mut value = self.insert_in_block(i, j, value);
        for block_idx in (i + 1)..self.n.div_ceil(B) {
            value = self.rotate_block(block_idx, value);
        }
    }

    /// Insert the given value at absolute index j in the i'th block.
    fn insert_in_block(&mut self, i: usize, mut j: usize, mut value: T) -> T {
        let head = self.head[i];
        if j >= head {
            // rotate the right half of the block.
            self.blocks[i][j..].rotate_right(1);
            value = replace(&mut self.blocks[i][j], value);
            j = 0;
        }
        if j < head {
            self.blocks[i][j..head].rotate_right(1);
            value = replace(&mut self.blocks[i][j], value);
        }
        value
    }

    /// Push a new smallest value to a block and return the largest value that doesn't fit anymore.
    fn rotate_block(&mut self, block_idx: usize, value: T) -> T {
        let head = &mut self.head[block_idx];
        *head = (*head + B - 1) % B;
        replace(&mut self.blocks[block_idx][*head], value)
    }
}
