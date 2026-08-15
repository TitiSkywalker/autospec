//! k-th factor (divisor) of `n`, or `-1` if there are fewer than `k` divisors.
//! Source: TheAlgorithms/Rust `src/number_theory/kth_factor.rs`.

/// Returns the `k`-th smallest positive divisor of `n` (1-indexed), or `-1`
/// if `n` has fewer than `k` divisors.
pub fn kth_factor(n: i64, k: i64) -> i64 {
    let mut count = 0i64;
    let mut i = 1i64;
    while i <= n {
        if n % i == 0 {
            count += 1;
            if count == k {
                return i;
            }
        }
        i += 1;
    }
    -1
}
