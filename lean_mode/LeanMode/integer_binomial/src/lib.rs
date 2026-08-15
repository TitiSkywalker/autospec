//! Binomial coefficient C(n, k).
//! Source: num-integer `src/lib.rs` (binomial) / TheAlgorithms/Rust `math/binomial_coefficient.rs`.

/// Computes the binomial coefficient `C(n, k) = n! / (k! · (n-k)!)` using the
/// multiplicative formula (each partial product is an integer, so each division
/// is exact).
pub fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    let mut res = 1u64;
    let mut i = 0u64;
    while i < k {
        res = (res * (n - i)) / (i + 1);
        i += 1;
    }
    res
}
