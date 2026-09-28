//! Sparse table for static range-minimum queries (RMQ).
//!
//! A flat `Vec<i64>` indexed `st[i * k + j]` where `k = floor(log2 n) + 1`.
//! `sparse_build` precomputes, in O(n log n), the minimum of every power-of-two
//! aligned range; `range_min(l, r)` answers in O(1).

/// A sparse table over `0 .. n`.
pub struct SparseTable {
    pub st: Vec<i64>,
    pub n: usize,
    pub k: usize,
}

/// floor(log2 x) for x ≥ 1.
fn log2_floor(x: usize) -> usize {
    let mut y = x;
    let mut r = 0;
    while y > 1 {
        y = y / 2;
        r += 1;
    }
    r
}

/// 2^e computed iteratively.
fn pow2(e: usize) -> usize {
    let mut p = 1;
    let mut i = 0;
    while i < e {
        p = p * 2;
        i += 1;
    }
    p
}

/// Build a sparse table from `a`.
pub fn sparse_build(a: &[i64]) -> SparseTable {
    let n = a.len();
    let k = log2_floor(n) + 1;
    let total = n * k;
    let mut st: Vec<i64> = vec![0; total];

    // level 0
    let mut i: usize = 0;
    while i < n {
        let idx = i * k;
        st[idx] = a[i];
        i += 1;
    }

    // levels 1 .. k
    let mut j: usize = 1;
    while j < k {
        let half = pow2(j - 1);
        let mut i: usize = 0;
        while i + half < n {
            let idx = i * k + j;
            let idx1 = i * k + (j - 1);
            let idx2 = (i + half) * k + (j - 1);
            let a1 = st[idx1];
            let a2 = st[idx2];
            let m = if a1 < a2 { a1 } else { a2 };
            st[idx] = m;
            i += 1;
        }
        j += 1;
    }

    SparseTable { st, n, k }
}

/// Flat index for `[row][col]` in a `k`-column layout.
fn flat_index(row: usize, k: usize, col: usize) -> usize {
    let t = row * k;
    let idx = t + col;
    idx
}

/// Minimum of `a[l..=r]` (inclusive).
pub fn range_min(st: &SparseTable, l: usize, r: usize) -> i64 {
    let k = st.k;
    let len = r - l + 1;
    let j = log2_floor(len);
    let pw = pow2(j);
    let i2 = r + 1 - pw;
    let idx1 = flat_index(l, k, j);
    let idx2 = flat_index(i2, k, j);
    let a1 = st.st[idx1];
    let a2 = st.st[idx2];
    if a1 < a2 { a1 } else { a2 }
}
