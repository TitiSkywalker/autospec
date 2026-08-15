#[test]
fn test_mod_pow_1() {
    assert_eq!(integer_mod_pow::mod_pow(2, 3, 5), 3);
}

#[test]
fn test_mod_pow_2() {
    assert_eq!(integer_mod_pow::mod_pow(7, 2, 13), 10);
}

#[test]
fn test_mod_pow_3() {
    assert_eq!(integer_mod_pow::mod_pow(5, 5, 31), 25);
}

#[test]
fn test_mod_pow_4() {
    assert_eq!(integer_mod_pow::mod_pow(10, 8, 11), 1);
}

#[test]
fn test_mod_pow_5() {
    assert_eq!(integer_mod_pow::mod_pow(123, 45, 67), 62);
}
