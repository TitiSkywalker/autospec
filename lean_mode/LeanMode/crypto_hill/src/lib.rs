//! Hill cipher (2×2 matrix) on the 26-letter alphabet — pure-integer implementation.
//!
//! The Hill cipher is a polygraphic substitution cipher based on linear algebra:
//!   encryption  c = K·p mod 26
//!   decryption  p = K⁻¹·c mod 26,   K⁻¹ = det(K)⁻¹ · adj(K)
//! where `K` is a 2×2 key matrix whose determinant must be coprime to 26
//! (`gcd(det(K), 26) = 1`) so that `K` is invertible mod 26.
//!
//! This testcase bridges linear algebra, number theory, and crypto: the
//! roundtrip `decrypt(encrypt(p)) = p` holds iff the determinant is a unit
//! modulo 26.  Under the `int` model, `i64` → `Int` (signed; the adjugate
//! entries `-k12`, `-k21` are genuinely negative).

/// Size of the alphabet (the modulus).
const ALPHABET: i64 = 26;

/// Euclidean reduction mod 26 (result in `0 .. 26`), handling negative inputs.
fn mod26(x: i64) -> i64 {
    let mut r = x;
    while r < 0 {
        r = r + ALPHABET;
    }
    while r >= ALPHABET {
        r = r - ALPHABET;
    }
    r
}

/// Modular multiplicative inverse of `a` modulo `m` (extended Euclidean
/// algorithm, signed Bézout coefficients).
///
/// Requires `gcd(a, m) = 1`; returns `x` with `a·x ≡ 1 (mod m)`, `0 ≤ x < m`.
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

/// Determinant of the 2×2 key matrix `[[k11, k12], [k21, k22]]`.
pub fn determinant(k11: i64, k12: i64, k21: i64, k22: i64) -> i64 {
    k11 * k22 - k12 * k21
}

/// Hill encryption of a 2-vector `(p1, p2)` (0 ≤ pi < 26) with key matrix
/// `[[k11, k12], [k21, k22]]`: `c = K·p mod 26`.
pub fn hill_encrypt(p1: i64, p2: i64, k11: i64, k12: i64, k21: i64, k22: i64) -> (i64, i64) {
    assert!(0 <= p1);
    assert!(p1 < ALPHABET);
    assert!(0 <= p2);
    assert!(p2 < ALPHABET);
    let c1 = mod26(k11 * p1 + k12 * p2);
    let c2 = mod26(k21 * p1 + k22 * p2);
    (c1, c2)
}

/// Hill decryption of a 2-vector `(c1, c2)`: `p = K⁻¹·c mod 26`, where
/// `K⁻¹ = det(K)⁻¹ · adj(K)`.
pub fn hill_decrypt(c1: i64, c2: i64, k11: i64, k12: i64, k21: i64, k22: i64) -> (i64, i64) {
    let det = determinant(k11, k12, k21, k22);
    let det_norm = mod26(det);
    let det_inv = mod_inverse(det_norm, ALPHABET);
    // adjugate of [[k11, k12], [k21, k22]] = [[k22, -k12], [-k21, k11]] (mod 26)
    let a11 = mod26(k22);
    let a12 = mod26(0 - k12);
    let a21 = mod26(0 - k21);
    let a22 = mod26(k11);
    let p1 = mod26(det_inv * (a11 * c1 + a12 * c2));
    let p2 = mod26(det_inv * (a21 * c1 + a22 * c2));
    (p1, p2)
}
