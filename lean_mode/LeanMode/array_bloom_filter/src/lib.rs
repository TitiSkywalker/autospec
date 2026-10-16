//! Bloom filter — a probabilistic set with no false negatives.
//!
//! An array of `size` bits together with `k` hash functions.  The `k` bit
//! positions of a key are derived by double hashing `(x + i·(x+1)) mod size`
//! for `i ∈ [0, k)`.  Insertion sets all `k` positions; a query answers `true`
//! iff all `k` positions are set.  False positives are possible, false
//! negatives are not.

/// A bloom filter with `k` hash functions over `size` bits.
pub struct BloomFilter {
    pub bits: Vec<bool>,
    pub k: usize,
}

/// Create an empty filter (`size > 0`).
pub fn bf_new(size: usize, k: usize) -> BloomFilter {
    let mut bits: Vec<bool> = vec![false; size];
    BloomFilter { bits, k }
}

/// `x mod sz` via repeated subtraction (exact for `sz > 0`).
fn mod_sz(x: usize, sz: usize) -> usize {
    let mut r = x;
    while r >= sz {
        r -= sz;
    }
    r
}

/// Insert `x`: set all `k` bit positions of `x`.
pub fn bf_insert(bf: BloomFilter, x: usize) -> BloomFilter {
    let mut r = bf;
    let sz = r.bits.len();
    let mut i: usize = 0;
    while i < r.k {
        let h = x + i * (x + 1);
        let idx = mod_sz(h, sz);
        r.bits[idx] = true;
        i += 1;
    }
    r
}

/// Query `x`: `true` iff all `k` bit positions of `x` are set.
pub fn bf_contains(bf: &BloomFilter, x: usize) -> bool {
    let sz = bf.bits.len();
    let mut i: usize = 0;
    let mut present = true;
    while i < bf.k {
        let h = x + i * (x + 1);
        let idx = mod_sz(h, sz);
        if !bf.bits[idx] {
            present = false;
        }
        i += 1;
    }
    present
}
