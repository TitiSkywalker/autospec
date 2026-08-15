//! Primality test via trial division (odd divisors only).
//! Source: TheAlgorithms/Rust `src/math/prime_check.rs` / num-prime `src/nt_funcs.rs` (is_prime64).

/// Returns `true` iff `n` is prime.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3u64;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}
