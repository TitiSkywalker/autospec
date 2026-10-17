//! Factorial of a non-negative integer.
//! Source: TheAlgorithms/Rust `src/math/factorial.rs`.

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
