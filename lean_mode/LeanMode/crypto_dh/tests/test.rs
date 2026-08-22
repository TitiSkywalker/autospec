// DH test vectors — small primes for easy verification.

// --- mod_pow tests ---

#[test]
fn test_mod_pow_1() {
    assert_eq!(crypto_dh::mod_pow(2, 3, 5), 3);
}

#[test]
fn test_mod_pow_2() {
    assert_eq!(crypto_dh::mod_pow(5, 6, 23), 8);
}

// --- DH public-key tests ---

#[test]
fn test_dh_public_key_1() {
    // p=23, g=5, a=6 → A = 5^6 mod 23 = 8
    assert_eq!(crypto_dh::dh_public_key(5, 6, 23), 8);
}

#[test]
fn test_dh_public_key_2() {
    // p=23, g=5, b=15 → B = 5^15 mod 23 = 19
    assert_eq!(crypto_dh::dh_public_key(5, 15, 23), 19);
}

// --- DH key agreement: Alice and Bob compute the same shared secret ---

#[test]
fn test_dh_agreement_1() {
    // p=23, g=5, a=6, b=15 → shared = 2
    let p = 23;
    let g = 5;
    let a = 6;
    let b = 15;
    let alice_public = crypto_dh::dh_public_key(g, a, p);
    let bob_public = crypto_dh::dh_public_key(g, b, p);
    let alice_shared = crypto_dh::dh_shared_secret(bob_public, a, p);
    let bob_shared = crypto_dh::dh_shared_secret(alice_public, b, p);
    assert_eq!(alice_shared, bob_shared);
    assert_eq!(alice_shared, 2);
}

#[test]
fn test_dh_agreement_2() {
    // p=97, g=5, a=7, b=11
    let p = 97;
    let g = 5;
    let a = 7;
    let b = 11;
    let alice_public = crypto_dh::dh_public_key(g, a, p);
    let bob_public = crypto_dh::dh_public_key(g, b, p);
    let alice_shared = crypto_dh::dh_shared_secret(bob_public, a, p);
    let bob_shared = crypto_dh::dh_shared_secret(alice_public, b, p);
    assert_eq!(alice_shared, bob_shared);
}

#[test]
fn test_dh_agreement_3() {
    // p=47, g=5, a=13, b=9
    let p = 47;
    let g = 5;
    let a = 13;
    let b = 9;
    let alice_public = crypto_dh::dh_public_key(g, a, p);
    let bob_public = crypto_dh::dh_public_key(g, b, p);
    let alice_shared = crypto_dh::dh_shared_secret(bob_public, a, p);
    let bob_shared = crypto_dh::dh_shared_secret(alice_public, b, p);
    assert_eq!(alice_shared, bob_shared);
}

// Symmetric: swapping private keys gives the same shared secret.
#[test]
fn test_dh_symmetric() {
    let p = 23;
    let g = 5;
    let a = 6;
    let b = 15;
    let shared_ab = crypto_dh::dh_shared_secret(crypto_dh::dh_public_key(g, a, p), b, p);
    let shared_ba = crypto_dh::dh_shared_secret(crypto_dh::dh_public_key(g, b, p), a, p);
    assert_eq!(shared_ab, shared_ba);
}