// RSA test vectors (classic example: p=61, q=53, n=3233, e=17, d=2753).

// --- gcd tests ---

#[test]
fn test_gcd_1() {
    assert_eq!(crypto_rsa::gcd(12, 8), 4);
}

#[test]
fn test_gcd_2() {
    assert_eq!(crypto_rsa::gcd(17, 3120), 1);
}

#[test]
fn test_gcd_3() {
    assert_eq!(crypto_rsa::gcd(1071, 462), 21);
}

// --- mod_inverse tests ---

#[test]
fn test_mod_inverse_1() {
    // 17⁻¹ mod 3120 = 2753
    assert_eq!(crypto_rsa::mod_inverse(17, 3120), 2753);
}

#[test]
fn test_mod_inverse_2() {
    // 3⁻¹ mod 11 = 4 (since 3*4=12≡1 mod 11)
    assert_eq!(crypto_rsa::mod_inverse(3, 11), 4);
}

#[test]
fn test_mod_inverse_3() {
    // 7⁻¹ mod 15 = 13 (since 7*13=91≡1 mod 15)
    assert_eq!(crypto_rsa::mod_inverse(7, 15), 13);
}

// --- mod_pow tests ---

#[test]
fn test_mod_pow_1() {
    assert_eq!(crypto_rsa::mod_pow(2, 3, 5), 3);
}

#[test]
fn test_mod_pow_2() {
    assert_eq!(crypto_rsa::mod_pow(7, 2, 13), 10);
}

#[test]
fn test_mod_pow_3() {
    // 65^17 mod 3233 = 2790 (classic RSA example)
    assert_eq!(crypto_rsa::mod_pow(65, 17, 3233), 2790);
}

// --- RSA encrypt/decrypt tests (n=3233, e=17, d=2753) ---

#[test]
fn test_rsa_encrypt_known() {
    // Classic RSA example: m=65 → c=2790
    assert_eq!(crypto_rsa::rsa_encrypt(65, 17, 3233), 2790);
}

#[test]
fn test_rsa_decrypt_known() {
    // Classic RSA example: c=2790 → m=65
    assert_eq!(crypto_rsa::rsa_decrypt(2790, 2753, 3233), 65);
}

#[test]
fn test_rsa_roundtrip_1() {
    let n = 3233;
    let e = 17;
    let d = 2753;
    let m = 42;
    let c = crypto_rsa::rsa_encrypt(m, e, n);
    let back = crypto_rsa::rsa_decrypt(c, d, n);
    assert_eq!(back, m);
}

#[test]
fn test_rsa_roundtrip_2() {
    let n = 3233;
    let e = 17;
    let d = 2753;
    let m = 123;
    let c = crypto_rsa::rsa_encrypt(m, e, n);
    let back = crypto_rsa::rsa_decrypt(c, d, n);
    assert_eq!(back, m);
}

#[test]
fn test_rsa_roundtrip_3() {
    let n = 3233;
    let e = 17;
    let d = 2753;
    let m = 999;
    let c = crypto_rsa::rsa_encrypt(m, e, n);
    let back = crypto_rsa::rsa_decrypt(c, d, n);
    assert_eq!(back, m);
}

// --- RSA with a second key pair (p=11, q=13, n=143, φ=120, e=7, d=103) ---

#[test]
fn test_rsa_roundtrip_small() {
    let n = 143;
    let e = 7;
    let d = 103;
    let m = 5;
    let c = crypto_rsa::rsa_encrypt(m, e, n);
    let back = crypto_rsa::rsa_decrypt(c, d, n);
    assert_eq!(back, m);
}

#[test]
fn test_rsa_encrypt_small() {
    // 8^7 mod 143 = 57
    assert_eq!(crypto_rsa::rsa_encrypt(8, 7, 143), 57);
}

#[test]
fn test_rsa_decrypt_small() {
    // 57^103 mod 143 = 8
    assert_eq!(crypto_rsa::rsa_decrypt(57, 103, 143), 8);
}