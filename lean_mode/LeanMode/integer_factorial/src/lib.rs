//! Factorial of a non-negative integer.

/// Computes `n!` iteratively.
pub fn factorial(n: u64) -> u64 {
    let mut result = 1;
    let mut i = 1;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    result
}
