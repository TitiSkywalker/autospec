//! Euler's totient function φ(n).

/// Computes φ(n) = n * Π_{p|n} (1 - 1/p) via trial division.
pub fn euler_totient(n: u64) -> u64 {
    let mut result = n;
    let mut num = n;
    let mut p = 2;
    while p * p <= num {
        if num % p == 0 {
            while num % p == 0 {
                num = num / p;
            }
            result = result - result / p;
        }
        p = p + 1;
    }
    if num > 1 {
        result = result - result / num;
    }
    result
}
