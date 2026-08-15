//! Integer n-th root (⌊ⁿ√x⌋) via Newton's method.
//! Source: num-integer `src/roots.rs` (Roots::nth_root).

/// `x / base^exp` computed by repeated integer division (never overflows).
fn idiv_pow(x: u64, base: u64, exp: u64) -> u64 {
    let mut result = x;
    let mut i = 0u64;
    while i < exp {
        result /= base;
        i += 1;
    }
    result
}

/// Computes `⌊ⁿ√x⌋`, the largest `r` with `rⁿ ≤ x < (r+1)ⁿ`.
/// Requires `n >= 1`.
pub fn nth_root(x: u64, n: u64) -> u64 {
    assert!(n >= 1);
    if n == 1 {
        return x;
    }
    if x < 2 {
        return x; // 0 and 1
    }
    let n1 = n - 1;
    let mut r = x;
    let mut next = (idiv_pow(x, r, n1) + r * n1) / n;
    while next < r {
        r = next;
        next = (idiv_pow(x, r, n1) + r * n1) / n;
    }
    r
}
