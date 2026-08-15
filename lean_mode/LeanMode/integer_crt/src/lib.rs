//! Chinese Remainder Theorem.
//! Source: TheAlgorithms/Rust `src/math/chinese_remainder_theorem.rs`.

/// Greatest common divisor of two non-negative integers (Euclid).
fn gcd(a: i64, b: i64) -> i64 {
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    x
}

/// Modular inverse of `a` modulo `m` (assumes `gcd(a, m) = 1`), via extended Euclid.
fn mod_inv(a: i64, m: i64) -> i64 {
    let mut old_r = a;
    let mut r = m;
    let mut old_s = 1i64;
    let mut s = 0i64;
    while r != 0 {
        let q = old_r / r;
        let tmp_r = r;
        r = old_r - q * r;
        old_r = tmp_r;
        let tmp_s = s;
        s = old_s - q * s;
        old_s = tmp_s;
    }
    // old_s * a ≡ gcd(a, m) = 1 (mod m); normalize to [0, m).
    ((old_s % m) + m) % m
}

/// Solves `x ≡ residues[i] (mod moduli[i])` for pairwise-coprime, positive moduli.
/// Requires a non-empty input and pairwise-coprime positive moduli (asserted).
pub fn crt(residues: &[i64], moduli: &[i64]) -> i64 {
    let len = residues.len();
    assert!(len > 0);

    // Product of all moduli.
    let mut prod = 1i64;
    let mut i: usize = 0;
    while i < len {
        assert!(moduli[i] > 0);
        prod *= moduli[i];
        i += 1;
    }

    // Pairwise coprime.
    let mut i: usize = 0;
    while i < len {
        let mut j: usize = i + 1;
        while j < len {
            assert!(gcd(moduli[i], moduli[j]) == 1);
            j += 1;
        }
        i += 1;
    }

    // Sum of residues[i] * (prod/moduli[i]) * inverse(prod/moduli[i], moduli[i]).
    let mut sum = 0i64;
    let mut i: usize = 0;
    while i < len {
        let p = prod / moduli[i];
        sum += residues[i] * mod_inv(p, moduli[i]) * p;
        i += 1;
    }

    ((sum % prod) + prod) % prod
}
