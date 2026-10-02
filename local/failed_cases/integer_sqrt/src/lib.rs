//! Integer square root — floor of √x via the Babylonian method.
//! Source: num-integer `src/roots.rs` (Roots::sqrt).

/// Computes `⌊√x⌋`, the largest `r` with `r² ≤ x < (r+1)²`.
pub fn isqrt(x: u64) -> u64 {
    if x < 2 {
        return x;
    }
    let mut r = x;
    let mut next = (x / r + r) / 2;
    while next < r {
        r = next;
        next = (x / r + r) / 2;
    }
    r
}
