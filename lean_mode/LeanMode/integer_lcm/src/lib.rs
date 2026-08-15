//! Least common multiple of two non-negative integers.
//! Source: num-integer `src/lib.rs` (lcm).

/// Computes `lcm(a, b) = a * b / gcd(a, b)`.
pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        0
    } else {
        let mut x = a;
        let mut y = b;
        while y != 0 {
            let t = y;
            y = x % y;
            x = t;
        }
        a / x * b
    }
}
