#[test]
fn test_pos_pos() {
    assert_eq!(integer_div_mod::div_floor(7, 3), 2);
    assert_eq!(integer_div_mod::mod_floor(7, 3), 1);
    assert_eq!(integer_div_mod::div_ceil(7, 3), 3);
}

#[test]
fn test_neg_pos() {
    assert_eq!(integer_div_mod::div_floor(-7, 3), -3);
    assert_eq!(integer_div_mod::mod_floor(-7, 3), 2);
    assert_eq!(integer_div_mod::div_ceil(-7, 3), -2);
}

#[test]
fn test_pos_neg() {
    assert_eq!(integer_div_mod::div_floor(7, -3), -3);
    assert_eq!(integer_div_mod::mod_floor(7, -3), -2);
    assert_eq!(integer_div_mod::div_ceil(7, -3), -2);
}

#[test]
fn test_neg_neg() {
    assert_eq!(integer_div_mod::div_floor(-7, -3), 2);
    assert_eq!(integer_div_mod::mod_floor(-7, -3), -1);
    assert_eq!(integer_div_mod::div_ceil(-7, -3), 3);
}

#[test]
fn test_exact_div() {
    assert_eq!(integer_div_mod::div_floor(6, 3), 2);
    assert_eq!(integer_div_mod::mod_floor(6, 3), 0);
    assert_eq!(integer_div_mod::div_ceil(6, 3), 2);
    assert_eq!(integer_div_mod::div_floor(-6, 3), -2);
    assert_eq!(integer_div_mod::mod_floor(-6, 3), 0);
    assert_eq!(integer_div_mod::div_ceil(-6, 3), -2);
}
