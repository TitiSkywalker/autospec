//! RSA encryption and decryption — pure-integer implementation.
//!
//! Implements the RSA public-key cryptosystem: key generation from primes
//! `p, q`, modular exponentiation (`mod_pow`), and encrypt/decrypt.
//! Under the `int` model, `u64`/`i64` → `Int` (unbounded).

/// Euclidean algorithm: `gcd(a, b)`.
pub fn gcd(a: u64, b: u64) -> u64 {
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    x
}

/// Modular multiplicative inverse of `a` modulo `m` (extended Euclidean algorithm).
///
/// Requires `gcd(a, m) = 1`.  Returns `x` such that `a * x ≡ 1 (mod m)`
/// with `0 ≤ x < m`.
pub fn mod_inverse(a: i64, m: i64) -> i64 {
    let mut old_r = a;
    let mut r = m;
    let mut old_s = 1;
    let mut s = 0;
    while r != 0 {
        let q = old_r / r;
        let next_r = old_r - q * r;
        old_r = r;
        r = next_r;
        let next_s = old_s - q * s;
        old_s = s;
        s = next_s;
    }
    assert!(old_r == 1);
    (old_s % m + m) % m
}

/// Modular exponentiation: `base^exp mod modulus` (square-and-multiply).
///
/// Requires `modulus > 0`.
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

/// RSA encryption: `ciphertext = message^e mod n`.
pub fn rsa_encrypt(message: u64, e: u64, n: u64) -> u64 {
    mod_pow(message, e, n)
}

/// RSA decryption: `message = ciphertext^d mod n`.
pub fn rsa_decrypt(ciphertext: u64, d: u64, n: u64) -> u64 {
    mod_pow(ciphertext, d, n)
}