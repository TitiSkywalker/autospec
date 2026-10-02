//! Batch Euler totient: φ(k) for all k = 1..n.
//! Source: TheAlgorithms/Rust `src/number_theory/compute_totient.rs`.

/// Returns `[φ(1), φ(2), ..., φ(n)]` using the sieve of the totient function.
pub fn euler_totient_sieve(n: usize) -> Vec<usize> {
    // phi[i] = i initially.
    let mut phi: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i <= n {
        phi.push(i);
        i += 1;
    }

    // For each prime p, scale down its multiples.
    let mut p: usize = 2;
    while p <= n {
        if phi[p] == p {
            // p is prime: φ(p) = p - 1.
            let pm1 = p - 1;
            phi[p] = pm1;
            let mut m: usize = 2 * p;
            while m <= n {
                let val = phi[m];
                let q = val / p;
                let new_val = q * pm1;
                phi[m] = new_val;
                m += p;
            }
        }
        p += 1;
    }

    // Drop phi[0]; return phi[1..=n].
    let mut result: Vec<usize> = Vec::new();
    let mut i: usize = 1;
    while i <= n {
        result.push(phi[i]);
        i += 1;
    }
    result
}
