//! Prime factorization via trial division (with repetition, increasing order).
//! Source: TheAlgorithms/Rust `src/math/prime_factors.rs` / num-prime `src/nt_funcs.rs` (factorize64).

/// Returns the prime factors of `n` in non-decreasing order, with repetition
/// (so their product equals `n`). Requires `n >= 1`.
pub fn prime_factors(n: u64) -> Vec<u64> {
    assert!(n >= 1);
    let mut factors: Vec<u64> = Vec::new();
    let mut m = n;
    let mut p = 2u64;
    while p * p <= m {
        while m % p == 0 {
            factors.push(p);
            m /= p;
        }
        p += 1;
    }
    if m > 1 {
        factors.push(m);
    }
    factors
}
