// Hill cipher (2×2, mod 26) test vectors.

// --- mod_inverse ---

#[test]
fn test_mod_inverse_1() {
    // 7⁻¹ mod 26 = 15 (7·15 = 105 ≡ 1)
    assert_eq!(crypto_hill::mod_inverse(7, 26), 15);
}

#[test]
fn test_mod_inverse_2() {
    // 9⁻¹ mod 26 = 3 (9·3 = 27 ≡ 1)
    assert_eq!(crypto_hill::mod_inverse(9, 26), 3);
}

#[test]
fn test_mod_inverse_3() {
    // 3⁻¹ mod 26 = 9 (3·9 = 27 ≡ 1)
    assert_eq!(crypto_hill::mod_inverse(3, 26), 9);
}

// --- determinant ---

#[test]
fn test_determinant_1() {
    assert_eq!(crypto_hill::determinant(2, 5, 1, 6), 7);
}

#[test]
fn test_determinant_2() {
    assert_eq!(crypto_hill::determinant(3, 3, 2, 5), 9);
}

// --- hill_encrypt (key [[2,5],[1,6]], det = 7) ---

#[test]
fn test_encrypt_1() {
    // c = [[2,5],[1,6]] · [7,4] = (34, 31) ≡ (8, 5)
    let (c1, c2) = crypto_hill::hill_encrypt(7, 4, 2, 5, 1, 6);
    assert_eq!(c1, 8);
    assert_eq!(c2, 5);
}

#[test]
fn test_encrypt_2() {
    let (c1, c2) = crypto_hill::hill_encrypt(0, 0, 2, 5, 1, 6);
    assert_eq!(c1, 0);
    assert_eq!(c2, 0);
}

#[test]
fn test_encrypt_3() {
    // (2·1+5·1, 1·1+6·1) = (7, 7)
    let (c1, c2) = crypto_hill::hill_encrypt(1, 1, 2, 5, 1, 6);
    assert_eq!(c1, 7);
    assert_eq!(c2, 7);
}

// --- hill_decrypt (key [[2,5],[1,6]], K⁻¹ = [[12,3],[11,4]]) ---

#[test]
fn test_decrypt_1() {
    let (p1, p2) = crypto_hill::hill_decrypt(8, 5, 2, 5, 1, 6);
    assert_eq!(p1, 7);
    assert_eq!(p2, 4);
}

#[test]
fn test_decrypt_2() {
    let (p1, p2) = crypto_hill::hill_decrypt(0, 0, 2, 5, 1, 6);
    assert_eq!(p1, 0);
    assert_eq!(p2, 0);
}

// --- roundtrip (decrypt(encrypt(p)) = p), key [[2,5],[1,6]] ---

#[test]
fn test_roundtrip_1() {
    let (c1, c2) = crypto_hill::hill_encrypt(7, 4, 2, 5, 1, 6);
    let (p1, p2) = crypto_hill::hill_decrypt(c1, c2, 2, 5, 1, 6);
    assert_eq!(p1, 7);
    assert_eq!(p2, 4);
}

#[test]
fn test_roundtrip_2() {
    let (c1, c2) = crypto_hill::hill_encrypt(11, 11, 2, 5, 1, 6);
    let (p1, p2) = crypto_hill::hill_decrypt(c1, c2, 2, 5, 1, 6);
    assert_eq!(p1, 11);
    assert_eq!(p2, 11);
}

#[test]
fn test_roundtrip_3() {
    let (c1, c2) = crypto_hill::hill_encrypt(25, 24, 2, 5, 1, 6);
    let (p1, p2) = crypto_hill::hill_decrypt(c1, c2, 2, 5, 1, 6);
    assert_eq!(p1, 25);
    assert_eq!(p2, 24);
}

// --- roundtrip with second key [[3,3],[2,5]], det = 9 ---

#[test]
fn test_roundtrip_key2() {
    let (c1, c2) = crypto_hill::hill_encrypt(1, 0, 3, 3, 2, 5);
    let (p1, p2) = crypto_hill::hill_decrypt(c1, c2, 3, 3, 2, 5);
    assert_eq!(p1, 1);
    assert_eq!(p2, 0);
}

#[test]
fn test_roundtrip_key2_b() {
    let (c1, c2) = crypto_hill::hill_encrypt(19, 22, 3, 3, 2, 5);
    let (p1, p2) = crypto_hill::hill_decrypt(c1, c2, 3, 3, 2, 5);
    assert_eq!(p1, 19);
    assert_eq!(p2, 22);
}
