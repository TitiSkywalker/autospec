//! Sieve of Eratosthenes: all primes ≤ `n`.
//! Source: TheAlgorithms/Rust `src/math/sieve_of_eratosthenes.rs` / num-prime `src/nt_funcs.rs` (primes).

/// Returns all primes ≤ `n` in increasing order.
pub fn prime_sieve(n: usize) -> Vec<usize> {
    let mut primes: Vec<usize> = Vec::new();
    if n < 2 {
        return primes;
    }
    let mut is_prime: Vec<bool> = Vec::new();
    let mut i: usize = 0;
    while i <= n {
        is_prime.push(true);
        i += 1;
    }
    is_prime[0] = false;
    is_prime[1] = false;

    let mut p: usize = 2;
    while p <= n {
        if is_prime[p] {
            primes.push(p);
            if p <= n / p {
                // p * p <= n (avoids overflow)
                let mut j: usize = p * p;
                while j <= n {
                    is_prime[j] = false;
                    j += p;
                }
            }
        }
        p += 1;
    }
    primes
}
