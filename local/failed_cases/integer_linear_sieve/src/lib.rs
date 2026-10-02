//! Linear sieve: minimum prime factor for every k ≤ n, plus the primes ≤ n.
//! Source: TheAlgorithms/Rust `src/math/linear_sieve.rs`.

/// Returns `(primes, mpf)` where `primes` lists the primes ≤ `n` and
/// `mpf[k]` is the least prime factor of `k` (for `k ≥ 2`; `mpf[0] = mpf[1] = 0`).
pub fn linear_sieve(n: usize) -> (Vec<usize>, Vec<usize>) {
    let mut primes: Vec<usize> = Vec::new();
    let mut mpf: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i <= n {
        mpf.push(0);
        i += 1;
    }

    let mut i: usize = 2;
    while i <= n {
        if mpf[i] == 0 {
            mpf[i] = i;
            primes.push(i);
        }
        // For each prime p ≤ mpf[i], mark p·i with p.
        let mut k: usize = 0;
        while k < primes.len() {
            let p = primes[k];
            if p > mpf[i] || p > n / i {
                break;
            }
            mpf[p * i] = p;
            k += 1;
        }
        i += 1;
    }
    (primes, mpf)
}
