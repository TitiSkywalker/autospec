#[test]
fn test_div_ceil_1() {
    assert_eq!(integer_div_ceil::div_ceil_u64(7, 3), 3);
}

#[test]
fn test_div_ceil_exact() {
    assert_eq!(integer_div_ceil::div_ceil_u64(6, 3), 2);
}

#[test]
fn test_div_ceil_zero() {
    assert_eq!(integer_div_ceil::div_ceil_u64(0, 3), 0);
}

#[test]
fn test_div_ceil_one() {
    assert_eq!(integer_div_ceil::div_ceil_u64(1, 3), 1);
}

#[test]
fn test_div_ceil_large() {
    assert_eq!(integer_div_ceil::div_ceil_u64(100, 7), 15);
}
