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

#[cfg(test)]
mod tests {
    use super::TieredVec2;
    use fastrand::Rng;

    fn check<const B: usize>(tiered: &TieredVec2<i32, B, B>, vec: &[i32]) {
        assert_eq!(tiered.n, vec.len());

        for (idx, expected) in vec.iter().enumerate() {
            assert_eq!(
                *tiered.get(idx),
                *expected,
                "mismatch at index {idx} for block size {B}"
            );
        }
    }

    fn run_randomized_insert_get_test<const B: usize>() {
        let mut tiered = TieredVec2::<i32, B, B>::new();
        let mut vec = Vec::new();
        let mut rng = Rng::with_seed(0x1234_5678_9abc_def0);
        let capacity = B * B;

        for _ in 0..capacity {
            let idx = rng.usize(..=vec.len());
            let value = rng.i32(..);
            tiered.insert(idx, value);
            vec.insert(idx, value);

            check(&tiered, &vec);
        }

        check(&tiered, &vec);
    }

    #[test]
    fn tiered_vec2_randomized_insert_and_get_match_vec_model() {
        run_randomized_insert_get_test::<4>();
        run_randomized_insert_get_test::<8>();
        run_randomized_insert_get_test::<16>();
        run_randomized_insert_get_test::<32>();
        run_randomized_insert_get_test::<64>();
        run_randomized_insert_get_test::<128>();
    }
}
