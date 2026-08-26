#![feature(vec_from_fn)]

use std::mem::replace;

fn main() {
    println!("Hello, world!");
}

/// 2 level structure:
/// A blocks of size B
pub struct TieredVec2<T, const A: usize, const B: usize> {
    /// Current number of elements
    n: usize,

    /// Index < B of the first element of each block.
    head: [usize; A],
    /// The actual blocks of data.
    blocks: Vec<[T; B]>,
}

/// 3 level structure:
/// A superblocks of B blocks of size C
pub struct TieredVec3<T, const A: usize, const B: usize, const C: usize> {
    /// Current number of elements
    n: usize,

    /// Index < B*C of the first element of each superblock.
    super_head: [usize; A],
    /// Index < C of the first element of each block.
    head: Vec<[usize; B]>,
    /// The actual blocks of data.
    blocks: Vec<[[T; C]; B]>,
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
            blocks: Vec::from_fn(A, |_| std::array::from_fn(|_| T::default())),
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

impl<T: Default, const A: usize, const B: usize, const C: usize> TieredVec3<T, A, B, C> {
    pub fn new() -> Self {
        Self {
            n: 0,
            super_head: [0; A],
            head: Vec::from_fn(A, |_| [0; B]),
            blocks: Vec::from_fn(A, |_| {
                std::array::from_fn(|_| std::array::from_fn(|_| T::default()))
            }),
        }
    }

    fn block_index(&self, superblock_idx: usize, logical_block_idx: usize) -> usize {
        (self.super_head[superblock_idx] + logical_block_idx) % B
    }

    fn index(&self, mut idx: usize) -> (usize, usize, usize) {
        let i = idx / (B * C);
        idx = (idx + self.super_head[i]) % (B * C);
        let j = idx / C;
        idx = (idx + self.head[i][j]) % C;
        let k = idx;
        (i, j, k)
    }

    pub fn get(&self, idx: usize) -> &T {
        let (i, j, k) = self.index(idx);
        &self.blocks[i][j][k]
    }

    fn insert_index(&self, idx: usize) -> (usize, usize, usize, usize) {
        let i = idx / (B * C);
        let superblock_offset = idx % (B * C);
        let logical_block_idx = superblock_offset / C;
        let block_offset = superblock_offset % C;
        let block_idx = self.block_index(i, logical_block_idx);
        let elem_idx = (self.head[i][block_idx] + block_offset) % C;
        (i, logical_block_idx, block_idx, elem_idx)
    }

    fn superblock_count(&self) -> usize {
        self.n.div_ceil(B * C)
    }

    fn block_count(&self, i: usize) -> usize {
        let used = self.n.saturating_sub(i * B * C).min(B * C);
        used.div_ceil(C)
    }

    pub fn insert(&mut self, idx: usize, value: T) {
        assert!(idx <= self.n, "insert index out of bounds");
        assert!(self.n < A * B * C, "TieredVec is already full");
        self.n += 1;

        let (superblock_idx, logical_block_idx, block_idx, elem_idx) = self.insert_index(idx);
        let mut value = self.insert_in_block(superblock_idx, block_idx, elem_idx, value);

        for next_logical_block in (logical_block_idx + 1)..self.block_count(superblock_idx) {
            let next_block_idx = self.block_index(superblock_idx, next_logical_block);
            value = self.rotate_block(superblock_idx, next_block_idx, value);
        }

        for next_superblock_idx in (superblock_idx + 1)..self.superblock_count() {
            value = self.rotate_superblock(next_superblock_idx, value);
        }
    }

    fn insert_in_block(
        &mut self,
        superblock_idx: usize,
        block_idx: usize,
        mut elem_idx: usize,
        mut value: T,
    ) -> T {
        let head = self.head[superblock_idx][block_idx];
        if elem_idx >= head {
            self.blocks[superblock_idx][block_idx][elem_idx..].rotate_right(1);
            value = replace(&mut self.blocks[superblock_idx][block_idx][elem_idx], value);
            elem_idx = 0;
        }
        if elem_idx < head {
            self.blocks[superblock_idx][block_idx][elem_idx..head].rotate_right(1);
            value = replace(&mut self.blocks[superblock_idx][block_idx][elem_idx], value);
        }
        value
    }

    fn rotate_block(&mut self, i: usize, j: usize, value: T) -> T {
        let head = &mut self.head[i][j];
        *head = (*head + C - 1) % C;
        replace(&mut self.blocks[i][j][*head], value)
    }

    fn rotate_superblock(&mut self, i: usize, mut value: T) -> T {
        for logical_block_idx in 0..self.block_count(i) {
            let block_idx = self.block_index(i, logical_block_idx);
            value = self.rotate_block(i, block_idx, value);
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{TieredVec2, TieredVec3};
    use fastrand::Rng;
    use std::time::Instant;

    fn check2<const A: usize, const B: usize>(tiered: &TieredVec2<i32, A, B>, vec: &[i32]) {
        assert_eq!(tiered.n, vec.len());

        for (idx, expected) in vec.iter().enumerate() {
            assert_eq!(
                *tiered.get(idx),
                *expected,
                "TieredVec2 mismatch at index {idx} for block size {B}"
            );
        }
    }

    fn run2<const A: usize, const B: usize>() {
        let mut tiered = TieredVec2::<i32, A, B>::new();
        let mut vec = Vec::new();
        let mut rng = Rng::with_seed(0x1234_5678_9abc_def0);
        let capacity = A * B;

        for _ in 0..capacity {
            let idx = rng.usize(..=vec.len());
            let value = rng.i32(..);
            tiered.insert(idx, value);
            vec.insert(idx, value);
            check2(&tiered, &vec);
        }
    }

    #[test]
    fn tiered2() {
        run2::<2, 2>();
        run2::<2, 4>();
        run2::<4, 4>();
        run2::<16, 2>();
        run2::<2, 16>();
        run2::<32, 32>();
        run2::<128, 128>();
    }

    fn check3<const A: usize, const B: usize, const C: usize>(
        tiered: &TieredVec3<i32, A, B, C>,
        vec: &[i32],
    ) {
        assert_eq!(tiered.n, vec.len());

        for (idx, expected) in vec.iter().enumerate() {
            assert_eq!(
                *tiered.get(idx),
                *expected,
                "TieredVec3 mismatch at index {idx} for shape ({A}, {B}, {C})"
            );
        }
    }

    fn run3<const A: usize, const B: usize, const C: usize>() {
        let mut tiered = TieredVec3::<i32, A, B, C>::new();
        let mut vec = Vec::new();
        let mut rng = Rng::with_seed(0x9abc_def0_1234_5678);
        let capacity = A * B * C;

        for _ in 0..capacity {
            let idx = rng.usize(..=vec.len());
            let value = rng.i32(..);
            tiered.insert(idx, value);
            vec.insert(idx, value);
            check3(&tiered, &vec);
        }
    }

    #[test]
    fn tiered3() {
        run3::<2, 4, 4>();
        run3::<2, 4, 8>();
        run3::<2, 8, 4>();
        run3::<4, 4, 4>();
        run3::<32, 32, 32>();
    }

    fn run_scaling2<const A: usize, const B: usize>() {
        let mut tiered = TieredVec2::<i32, A, B>::new();
        let capacity = A * B;
        let mut rng = Rng::with_seed(0x9abc_def0_1234_5678);
        let start = Instant::now();

        for _ in 0..capacity {
            let idx = rng.usize(..=tiered.n);
            let value = rng.i32(..);
            tiered.insert(idx, value);
        }

        let duration = start.elapsed();
        let ns_per_insert = duration.as_nanos() as f64 / capacity as f64;
        let sqrt = (capacity as f64).sqrt();
        println!(
            "{A}x{B}: n={capacity}, sqrt(n)={sqrt:.2}, ns/insert={:.3}, ns/insert/sqrt(n)={:.6}",
            ns_per_insert,
            ns_per_insert / sqrt
        );
    }

    #[test]
    #[ignore = "benchmark-style scaling check"]
    fn scaling2() {
        run_scaling2::<32, 32>();
        run_scaling2::<64, 64>();
        run_scaling2::<128, 128>();
        run_scaling2::<256, 256>();
        run_scaling2::<512, 512>();
        run_scaling2::<1024, 1024>();
        run_scaling2::<2048, 2048>();
    }

    fn run_scaling3<const A: usize, const B: usize, const C: usize>() {
        let mut tiered = TieredVec3::<i32, A, B, C>::new();
        let capacity = A * B * C;
        let mut rng = Rng::with_seed(0x9abc_def0_1234_5678);
        let start = Instant::now();

        for _ in 0..capacity {
            let idx = rng.usize(..=tiered.n);
            let value = rng.i32(..);
            tiered.insert(idx, value);
        }

        let duration = start.elapsed();
        let ns_per_insert = duration.as_nanos() as f64 / capacity as f64;
        let cbrt = (capacity as f64).cbrt();
        println!(
            "{A}x{B}x{C}: n={capacity}, cbrt(n)={cbrt:.2}, ns/insert={:.3}, ns/insert/cbrt(n)={:.6}",
            ns_per_insert,
            ns_per_insert / cbrt
        );
    }

    #[test]
    #[ignore = "benchmark-style scaling check"]
    fn scaling3() {
        run_scaling3::<16, 16, 16>();
        run_scaling3::<32, 32, 32>();
        run_scaling3::<64, 64, 64>();
        run_scaling3::<64, 64, 128>();
        run_scaling3::<64, 128, 128>();
        run_scaling3::<128, 128, 128>();
        run_scaling3::<256, 256, 256>();
        run_scaling3::<512, 512, 512>();
    }
}
