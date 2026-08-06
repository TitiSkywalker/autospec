//! Pure GCD — computes the greatest common divisor using the Euclidean algorithm.

/// Computes the greatest common divisor of two `u64` values.
pub fn compute_gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}
