//! Modular multiplicative inverse — computes `b⁻¹ mod m` when `gcd(b, m) = 1`.

/// Returns the modular inverse of `b` modulo `m` (requires `gcd(b, m) = 1`).
///
/// The result is normalized to the range `0 .. m`.
pub fn mod_inverse(b: i64, m: i64) -> i64 {
    let mut old_r = b;
    let mut r = m;
    let mut old_s = 1;
    let mut s = 0;
    while r != 0 {
        let q = old_r / r;
        let next_r = old_r - q * r;
        old_r = r;
        r = next_r;
        let next_s = old_s - q * s;
        old_s = s;
        s = next_s;
    }
    // old_r = gcd(b, m); old_s is the Bézout coefficient of b.
    assert!(old_r == 1);
    (old_s % m + m) % m
}
