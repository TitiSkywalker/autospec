#[test]
fn test_zeros() {
    assert_eq!(integer_gcd_binary::gcd_binary(0, 0), 0);
    assert_eq!(integer_gcd_binary::gcd_binary(0, 7), 7);
    assert_eq!(integer_gcd_binary::gcd_binary(7, 0), 7);
}

#[test]
fn test_basic() {
    assert_eq!(integer_gcd_binary::gcd_binary(48, 18), 6);
    assert_eq!(integer_gcd_binary::gcd_binary(270, 192), 6);
    assert_eq!(integer_gcd_binary::gcd_binary(1071, 462), 21);
    assert_eq!(integer_gcd_binary::gcd_binary(17, 19), 1);
}

#[test]
fn test_powers_of_two() {
    assert_eq!(integer_gcd_binary::gcd_binary(8, 12), 4);
    assert_eq!(integer_gcd_binary::gcd_binary(16, 24), 8);
    assert_eq!(integer_gcd_binary::gcd_binary(2, 4), 2);
    assert_eq!(integer_gcd_binary::gcd_binary(1000000, 500), 500);
}
