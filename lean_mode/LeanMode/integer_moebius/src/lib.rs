//! Möbius function μ(n).
//!
//! μ(n) = 0 if n has a square factor, otherwise (-1)^k where k is the number
//! of distinct prime factors of n.

/// Computes the Möbius function of `n`.
pub fn moebius(n: u64) -> i64 {
    let mut num = n;
    let mut result = 1;
    let mut p = 2;
    while p * p <= num {
        if num % p == 0 {
            num = num / p;
            result = -result;
            if num % p == 0 {
                result = 0;
            }
        }
        p = p + 1;
    }
    if num > 1 {
        result = -result;
    }
    result
}
