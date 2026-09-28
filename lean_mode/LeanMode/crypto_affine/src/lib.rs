//! Affine cipher on the 26-letter alphabet — pure-integer implementation.
//!
//! The affine cipher is a monoalphabetic substitution cipher:
//!   encryption  E(x) = (a·x + b) mod 26
//!   decryption  D(y) = a⁻¹ · (y − b) mod 26
//! where `a` must be coprime to 26 (`gcd(a, 26) = 1`) so that the modular
//! inverse `a⁻¹ mod 26` exists.
//!
//! This testcase is the cleanest bridge between number theory and crypto:
//! the roundtrip `decrypt(encrypt(x)) = x` holds *iff* `a` is invertible
//! mod 26.  Under the `nat` model, `u64` → `Nat` (unbounded, non-negative).

/// Size of the alphabet (the modulus).
pub const ALPHABET: u64 = 26;

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

/// Modular multiplicative inverse of `a` modulo `m` (extended Euclidean
/// algorithm with modular reduction at each step, keeping every intermediate
/// value non-negative).
///
/// Requires `gcd(a, m) = 1`; returns `x` with `a·x ≡ 1 (mod m)`, `0 ≤ x < m`.
pub fn mod_inverse(a: u64, m: u64) -> u64 {
    let mut old_r = a;
    let mut r = m;
    let mut old_s = 1;
    let mut s = 0;
    while r != 0 {
        let q = old_r / r;
        let next_r = old_r - q * r;
        old_r = r;
        r = next_r;
        let qs = (q * s) % m;
        let next_s = (old_s + m - qs) % m;
        old_s = s;
        s = next_s;
    }
    assert!(old_r == 1);
    old_s
}

/// Reduce `x` to the range `0 .. 26` by repeated subtraction (stays non-negative).
fn mod26(x: u64) -> u64 {
    let mut r = x;
    while r >= ALPHABET {
        r = r - ALPHABET;
    }
    r
}

/// Affine encryption of a single character `x` (0 ≤ x < 26): `(a·x + b) mod 26`.
pub fn affine_encrypt(x: u64, a: u64, b: u64) -> u64 {
    assert!(x < ALPHABET);
    mod26(a * x + b)
}

/// Affine decryption of a single character `y` (0 ≤ y < 26): `a⁻¹·(y − b) mod 26`.
///
/// Written as `(y + 26 − b) · a⁻¹ mod 26` so every intermediate value stays
/// non-negative (no signed arithmetic needed).
pub fn affine_decrypt(y: u64, a: u64, b: u64) -> u64 {
    let a_inv = mod_inverse(a, ALPHABET);
    mod26((y + ALPHABET - b) * a_inv)
}
