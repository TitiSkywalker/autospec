//! All divisors of `n` in increasing order.
//! Source: TheAlgorithms/Rust `src/math/factors.rs`.

/// Returns all positive divisors of `n` in increasing order. Requires `n >= 1`.
pub fn divisors(n: u64) -> Vec<u64> {
    assert!(n >= 1);
    let mut small: Vec<u64> = Vec::new();
    let mut large: Vec<u64> = Vec::new();
    let mut d = 1u64;
    while d * d <= n {
        if n % d == 0 {
            small.push(d);
            if d != n / d {
                large.push(n / d);
            }
        }
        d += 1;
    }
    // `large` holds the co-divisors in decreasing order; append them reversed.
    let mut result = small;
    let mut i = large.len();
    while i > 0 {
        i -= 1;
        result.push(large[i]);
    }
    result
}
