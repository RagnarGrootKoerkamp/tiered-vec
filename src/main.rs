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
    blocks: Box<[[T; B]; A]>,
}

/// 3 level structure:
/// A superblocks of B blocks of size C
pub struct TieredVec3<T, const A: usize, const B: usize, const C: usize> {
    /// Current number of elements
    n: usize,

    /// Pointer to the first block of each superblock.
    super_head: [usize; A],
    /// Pointer to the first element of each block.
    head: [[usize; B]; A],
    /// The actual blocks of data.
    blocks: Box<[[[T; C]; B]; A]>,
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
            blocks: Box::new(std::array::from_fn(|_| std::array::from_fn(|_| T::default()))),
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
            head: std::array::from_fn(|_| [0; B]),
            blocks: Box::new(std::array::from_fn(|_| {
                std::array::from_fn(|_| std::array::from_fn(|_| T::default()))
            })),
        }
    }

    fn block_index(&self, superblock_idx: usize, logical_block_idx: usize) -> usize {
        (self.super_head[superblock_idx] + logical_block_idx) % B
    }

    fn index(&self, idx: usize) -> (usize, usize, usize) {
        let superblock_idx = idx / (B * C);
        let superblock_offset = idx % (B * C);
        let logical_block_idx = superblock_offset / C;
        let block_offset = superblock_offset % C;
        let block_idx = self.block_index(superblock_idx, logical_block_idx);
        let elem_idx = (self.head[superblock_idx][block_idx] + block_offset) % C;
        (superblock_idx, block_idx, elem_idx)
    }

    fn insert_index(&self, idx: usize) -> (usize, usize, usize, usize) {
        let superblock_idx = idx / (B * C);
        let superblock_offset = idx % (B * C);
        let logical_block_idx = superblock_offset / C;
        let block_offset = superblock_offset % C;
        let block_idx = self.block_index(superblock_idx, logical_block_idx);
        let elem_idx = (self.head[superblock_idx][block_idx] + block_offset) % C;
        (superblock_idx, logical_block_idx, block_idx, elem_idx)
    }

    fn superblock_count(&self) -> usize {
        self.n.div_ceil(B * C)
    }

    fn block_count(&self, superblock_idx: usize) -> usize {
        let used = self.n.saturating_sub(superblock_idx * B * C).min(B * C);
        used.div_ceil(C)
    }

    pub fn get(&self, idx: usize) -> &T {
        let (i, j, k) = self.index(idx);
        &self.blocks[i][j][k]
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

    fn rotate_block(&mut self, superblock_idx: usize, block_idx: usize, value: T) -> T {
        let head = &mut self.head[superblock_idx][block_idx];
        *head = (*head + C - 1) % C;
        replace(&mut self.blocks[superblock_idx][block_idx][*head], value)
    }

    fn rotate_superblock(&mut self, superblock_idx: usize, mut value: T) -> T {
        for logical_block_idx in 0..self.block_count(superblock_idx) {
            let block_idx = self.block_index(superblock_idx, logical_block_idx);
            value = self.rotate_block(superblock_idx, block_idx, value);
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{TieredVec2, TieredVec3};
    use fastrand::Rng;
    use std::{hint::black_box, thread, time::Instant};

    struct ScalingPoint {
        label: &'static str,
        capacity: usize,
        cbrt_n: f64,
        ns_per_index: f64,
    }

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

    fn run_randomized_insert_get_test2<const A: usize, const B: usize>() {
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

    fn run_randomized_insert_get_test3<const A: usize, const B: usize, const C: usize>() {
        let mut tiered = TieredVec3::<i32, A, B, C>::new();
        let mut vec = Vec::new();
        let mut rng = Rng::with_seed(
            0x9abc_def0_1234_5678 ^ A as u64 ^ ((B as u64) << 16) ^ ((C as u64) << 32),
        );
        let capacity = A * B * C;

        for _ in 0..capacity {
            let idx = rng.usize(..=vec.len());
            let value = rng.i32(..);
            tiered.insert(idx, value);
            vec.insert(idx, value);
            check3(&tiered, &vec);
        }
    }

    fn measure_index_scaling<const A: usize, const B: usize, const C: usize>(
        label: &'static str,
        samples: usize,
    ) -> ScalingPoint {
        let tiered = TieredVec3::<(), A, B, C>::new();
        let capacity = A * B * C;
        let mut state = 0x9e37_79b9_7f4a_7c15_u64 ^ capacity as u64;
        let start = Instant::now();

        for _ in 0..samples {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let idx = (state as usize) % capacity;
            black_box(black_box(&tiered).index(black_box(idx)));
        }

        ScalingPoint {
            label,
            capacity,
            cbrt_n: (capacity as f64).cbrt(),
            ns_per_index: start.elapsed().as_secs_f64() * 1e9 / samples as f64,
        }
    }

    #[test]
    fn tiered_vec2_randomized_insert_and_get_match_vec_model() {
        run_randomized_insert_get_test2::<2, 2>();
        run_randomized_insert_get_test2::<2, 4>();
        run_randomized_insert_get_test2::<4, 4>();
        run_randomized_insert_get_test2::<16, 2>();
        run_randomized_insert_get_test2::<2, 16>();
        run_randomized_insert_get_test2::<32, 32>();
    }

    #[test]
    fn tiered_vec3_randomized_insert_and_get_match_vec_model() {
        run_randomized_insert_get_test3::<2, 4, 4>();
        run_randomized_insert_get_test3::<2, 4, 8>();
        run_randomized_insert_get_test3::<2, 8, 4>();
        run_randomized_insert_get_test3::<4, 4, 4>();
    }

    #[test]
    #[ignore = "benchmark-style scaling check"]
    fn tiered_vec3_index_runtime_scales_with_cuberoot_n() {
        const SAMPLES: usize = 2_000_000;

        let points = thread::Builder::new()
            .name("tiered-vec3-scaling".into())
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                [
                    measure_index_scaling::<10, 10, 10>("1K", SAMPLES),
                    measure_index_scaling::<100, 100, 100>("1M", SAMPLES),
                    measure_index_scaling::<465, 465, 465>("100M-ish", SAMPLES),
                ]
            })
            .expect("failed to spawn scaling thread")
            .join()
            .expect("scaling thread panicked");

        for point in &points {
            println!(
                "{}: n={}, cbrt(n)={:.2}, ns/index={:.3}, ns/index/cbrt(n)={:.6}",
                point.label,
                point.capacity,
                point.cbrt_n,
                point.ns_per_index,
                point.ns_per_index / point.cbrt_n
            );
        }

        for pair in points.windows(2) {
            let prev = &pair[0];
            let next = &pair[1];
            let cbrt_ratio = (next.capacity as f64 / prev.capacity as f64).cbrt();
            let time_ratio = next.ns_per_index / prev.ns_per_index.max(f64::EPSILON);

            println!(
                "{} -> {}: time ratio {:.3}, cbrt ratio {:.3}",
                prev.label, next.label, time_ratio, cbrt_ratio
            );

            assert!(
                time_ratio <= cbrt_ratio * 8.0,
                "index runtime grew faster than the generous cbrt(n) bound: {} -> {}",
                prev.label,
                next.label
            );
        }
    }
}
