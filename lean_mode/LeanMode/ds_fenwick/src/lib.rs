//! Fenwick tree (binary-indexed tree) for prefix sums with point updates.
//!
//! A flat `Vec<i64>` indexed 1-based; `lowbit(i) = i & -i`.  `update` adds a
//! delta to a single position in O(log n); `query` returns the prefix sum
//! `a[0..=i]` in O(log n).

/// A Fenwick tree over `0 .. n`.
pub struct BIT {
    pub tree: Vec<i64>,
    pub n: usize,
}

/// Lowest set bit of `x`.
fn lowbit(x: usize) -> usize {
    x & (0usize.wrapping_sub(x))
}

/// Create an all-zero Fenwick tree over `0 .. n`.
pub fn bit_new(n: usize) -> BIT {
    let size = n + 1;
    BIT {
        tree: vec![0; size],
        n,
    }
}

/// Add `delta` to position `i` (0-based).
pub fn update(b: BIT, i: usize, delta: i64) -> BIT {
    let mut r = b;
    let mut idx = i + 1;
    while idx <= r.n {
        let t = r.tree[idx] + delta;
        r.tree[idx] = t;
        let lb = lowbit(idx);
        let nidx = idx + lb;
        idx = nidx;
    }
    r
}

/// Prefix sum `a[0..=i]` (0-based).
pub fn query(b: &BIT, i: usize) -> i64 {
    let mut sum = 0;
    let mut idx = i + 1;
    while idx > 0 {
        sum = sum + b.tree[idx];
        let lb = lowbit(idx);
        let nidx = idx - lb;
        idx = nidx;
    }
    sum
}
