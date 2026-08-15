//! Binary exponentiation — computes `a^e` in O(log e) multiplications.

/// Computes `a^e` using exponentiation by squaring.
pub fn pow(a: u64, e: u64) -> u64 {
    let mut result = 1;
    let mut base = a;
    let mut exp = e;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base;
        }
        exp = exp / 2;
        base = base * base;
    }
    result
}
