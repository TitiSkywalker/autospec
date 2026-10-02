//! Integer cube root (⌊∛x⌋) via Newton's method.
//! Source: num-integer `src/roots.rs` (Roots::cbrt).

/// Computes `⌊∛x⌋`, the largest `r` with `r³ ≤ x < (r+1)³`.
pub fn icbrt(x: u64) -> u64 {
    if x < 2 {
        return x;
    }
    let mut r = x;
    let mut next = (x / (r * r) + 2 * r) / 3;
    while next < r {
        r = next;
        next = (x / (r * r) + 2 * r) / 3;
    }
    r
}
