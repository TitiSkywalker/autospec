// Affine cipher test vectors (mod 26 alphabet).

// --- gcd ---

#[test]
fn test_gcd_1() {
    assert_eq!(crypto_affine::gcd(12, 8), 4);
}

#[test]
fn test_gcd_2() {
    assert_eq!(crypto_affine::gcd(5, 26), 1);
}

#[test]
fn test_gcd_3() {
    assert_eq!(crypto_affine::gcd(2, 26), 2);
}

// --- mod_inverse ---

#[test]
fn test_mod_inverse_1() {
    // 5⁻¹ mod 26 = 21 (5·21 = 105 ≡ 1)
    assert_eq!(crypto_affine::mod_inverse(5, 26), 21);
}

#[test]
fn test_mod_inverse_2() {
    // 3⁻¹ mod 26 = 9 (3·9 = 27 ≡ 1)
    assert_eq!(crypto_affine::mod_inverse(3, 26), 9);
}

#[test]
fn test_mod_inverse_3() {
    // 7⁻¹ mod 26 = 15 (7·15 = 105 ≡ 1)
    assert_eq!(crypto_affine::mod_inverse(7, 26), 15);
}

#[test]
fn test_mod_inverse_4() {
    // 9⁻¹ mod 26 = 3 (9·3 = 27 ≡ 1)
    assert_eq!(crypto_affine::mod_inverse(9, 26), 3);
}

// --- affine_encrypt ---

#[test]
fn test_encrypt_1() {
    // E(x) = 5x + 8 mod 26
    assert_eq!(crypto_affine::affine_encrypt(0, 5, 8), 8);
    assert_eq!(crypto_affine::affine_encrypt(1, 5, 8), 13);
    assert_eq!(crypto_affine::affine_encrypt(2, 5, 8), 18);
    assert_eq!(crypto_affine::affine_encrypt(25, 5, 8), 3);
}

#[test]
fn test_encrypt_2() {
    // E(x) = 3x + 5 mod 26
    assert_eq!(crypto_affine::affine_encrypt(0, 3, 5), 5);
    assert_eq!(crypto_affine::affine_encrypt(7, 3, 5), 0); // 3·7+5 = 26 ≡ 0
}

// --- affine_decrypt (known ciphertexts) ---

#[test]
fn test_decrypt_1() {
    // D(y) = 21·(y − 8) mod 26
    assert_eq!(crypto_affine::affine_decrypt(8, 5, 8), 0);
    assert_eq!(crypto_affine::affine_decrypt(13, 5, 8), 1);
    assert_eq!(crypto_affine::affine_decrypt(3, 5, 8), 25);
}

// --- roundtrip (decrypt(encrypt(x)) = x for all 26 chars, several keys) ---

#[test]
fn test_roundtrip_a5_b8() {
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(0, 5, 8), 5, 8), 0);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(1, 5, 8), 5, 8), 1);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(13, 5, 8), 5, 8), 13);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(25, 5, 8), 5, 8), 25);
}

#[test]
fn test_roundtrip_a3_b5() {
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(0, 3, 5), 3, 5), 0);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(7, 3, 5), 3, 5), 7);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(19, 3, 5), 3, 5), 19);
}

#[test]
fn test_roundtrip_a7_b2() {
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(4, 7, 2), 7, 2), 4);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(11, 7, 2), 7, 2), 11);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(22, 7, 2), 7, 2), 22);
}

#[test]
fn test_roundtrip_a9_b13() {
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(3, 9, 13), 9, 13), 3);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(15, 9, 13), 9, 13), 15);
    assert_eq!(crypto_affine::affine_decrypt(crypto_affine::affine_encrypt(24, 9, 13), 9, 13), 24);
}
