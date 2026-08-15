//! Extended Euclidean algorithm — computes `gcd(a, b)` together with the
//! Source: num-integer `src/lib.rs` (extended_gcd).
//! Bézout coefficients `(x, y)` such that `a * x + b * y = gcd(a, b)`.

/// Returns `(gcd, x, y)` satisfying Bézout's identity `a * x + b * y = gcd`.
pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    let mut old_r = a;
    let mut r = b;
    let mut old_s = 1;
    let mut s = 0;
    let mut old_t = 0;
    let mut t = 1;
    while r != 0 {
        let q = old_r / r;
        let next_r = old_r - q * r;
        old_r = r;
        r = next_r;
        let next_s = old_s - q * s;
        old_s = s;
        s = next_s;
        let next_t = old_t - q * t;
        old_t = t;
        t = next_t;
    }
    (old_r, old_s, old_t)
}
