//! Diffie-Hellman key exchange — pure-integer implementation.
//!
//! DH allows two parties to agree on a shared secret over an insecure channel.
//! Private keys `a`, `b` are passed as explicit parameters (not randomly generated).
//! Under the `nat` model, `u64` → `Nat`.

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

/// DH public-key generation: `A = g^a mod p`.
pub fn dh_public_key(g: u64, a: u64, p: u64) -> u64 {
    mod_pow(g, a, p)
}

/// DH shared-secret computation: `shared = peer_public^my_private mod p`.
pub fn dh_shared_secret(peer_public: u64, my_private: u64, p: u64) -> u64 {
    mod_pow(peer_public, my_private, p)
}