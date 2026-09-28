//! Iterative segment tree for range sums with point updates.
//!
//! A flat `Vec<i64>` of size `2 * size` (where `size` is the next power of two
//! ≥ n), built bottom-up.  `range_sum(l, r)` returns the inclusive-range sum in
//! O(log n); `point_update(i, v)` sets position `i` in O(log n).

/// An iterative segment tree over `0 .. n`.
pub struct SegTree {
    pub tree: Vec<i64>,
    pub size: usize,
}

/// Smallest power of two ≥ `n`.
fn next_pow2(n: usize) -> usize {
    let mut s = 1;
    while s < n {
        let t = s * 2;
        s = t;
    }
    s
}

/// Build a segment tree from `a`.
pub fn segtree_build(a: &[i64]) -> SegTree {
    let n = a.len();
    let size = next_pow2(n);
    let total = size * 2;
    let mut tree: Vec<i64> = vec![0; total];
    let mut i: usize = 0;
    while i < n {
        let idx = size + i;
        tree[idx] = a[i];
        i += 1;
    }
    let mut j: usize = size - 1;
    while j > 0 {
        let l = 2 * j;
        let r = 2 * j + 1;
        let t = tree[l] + tree[r];
        tree[j] = t;
        j -= 1;
    }
    SegTree { tree, size }
}

/// Set position `i` to `v`.
pub fn point_update(st: SegTree, i: usize, v: i64) -> SegTree {
    let mut r = st;
    let mut idx = r.size + i;
    r.tree[idx] = v;
    let mut p = idx / 2;
    while p > 0 {
        let l = 2 * p;
        let rr = 2 * p + 1;
        let t = r.tree[l] + r.tree[rr];
        r.tree[p] = t;
        let np = p / 2;
        p = np;
    }
    r
}

/// Sum of `a[l..=r]` (inclusive).
pub fn range_sum(st: &SegTree, l: usize, r: usize) -> i64 {
    let mut sum = 0;
    let mut li = st.size + l;
    let mut ri = st.size + r;
    while li <= ri {
        let oddl = li % 2;
        if oddl == 1 {
            sum = sum + st.tree[li];
            li = li + 1;
        }
        let oddr = ri % 2;
        if oddr == 0 {
            sum = sum + st.tree[ri];
            ri = ri - 1;
        }
        li = li / 2;
        ri = ri / 2;
    }
    sum
}
