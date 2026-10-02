#[test]
fn test_inverse_1() {
    assert_eq!(integer_mod_inverse::mod_inverse(7, 13), 2);
}

#[test]
fn test_inverse_2() {
    assert_eq!(integer_mod_inverse::mod_inverse(5, 31), 25);
}

#[test]
fn test_inverse_3() {
    assert_eq!(integer_mod_inverse::mod_inverse(10, 11), 10);
}

#[test]
fn test_inverse_4() {
    assert_eq!(integer_mod_inverse::mod_inverse(123, 67), 6);
}

#[test]
fn test_inverse_5() {
    assert_eq!(integer_mod_inverse::mod_inverse(9, 17), 2);
}
