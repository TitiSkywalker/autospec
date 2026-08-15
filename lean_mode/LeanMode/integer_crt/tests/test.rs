#[test]
fn test_basic() {
    // x ≡ 2 (mod 3), x ≡ 3 (mod 5), x ≡ 1 (mod 7) → x = 8
    assert_eq!(integer_crt::crt(&[2, 3, 1], &[3, 5, 7]), 8);
    // x ≡ 1 (mod 3), x ≡ 4 (mod 5), x ≡ 6 (mod 7) → x = 34
    assert_eq!(integer_crt::crt(&[1, 4, 6], &[3, 5, 7]), 34);
}

#[test]
fn test_two_moduli() {
    // x ≡ 2 (mod 3), x ≡ 3 (mod 5) → x ≡ 8 (mod 15)
    assert_eq!(integer_crt::crt(&[2, 3], &[3, 5]), 8);
    // x ≡ 1 (mod 2), x ≡ 2 (mod 3), x ≡ 3 (mod 5) → x ≡ 23 (mod 30)
    assert_eq!(integer_crt::crt(&[1, 2, 3], &[2, 3, 5]), 23);
}
