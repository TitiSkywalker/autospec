//! Modular exponentiation — computes `base^exp mod modulus` (square-and-multiply).

/// Computes `base^exp mod modulus` for `modulus > 0`.
pub fn mod_pow(base: u64, exp: u64, modulus: u64) -> u64 {
    assert!(modulus > 0);
    let mut result = 1;
    let mut b = base % modulus;
    let mut e = exp;
    while e > 0 {
        if e % 2 == 1 {
            result = (result * b) % modulus;
        }
        e = e / 2;
        b = (b * b) % modulus;
    }
    result
}
