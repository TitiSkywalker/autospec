#[test]
fn test_pow_1() {
    assert_eq!(integer_pow::pow(2, 3), 8);
}

#[test]
fn test_pow_2() {
    assert_eq!(integer_pow::pow(4, 12), 16777216);
}

#[test]
fn test_pow_3() {
    assert_eq!(integer_pow::pow(6, 12), 2176782336);
}

#[test]
fn test_pow_4() {
    assert_eq!(integer_pow::pow(10, 4), 10000);
}

#[test]
fn test_pow_5() {
    assert_eq!(integer_pow::pow(20, 3), 8000);
}

#[test]
fn test_pow_6() {
    assert_eq!(integer_pow::pow(3, 21), 10460353203);
}

#[test]
fn test_pow_zero() {
    assert_eq!(integer_pow::pow(0, 0), 1);
    assert_eq!(integer_pow::pow(5, 0), 1);
    assert_eq!(integer_pow::pow(0, 5), 0);
}
