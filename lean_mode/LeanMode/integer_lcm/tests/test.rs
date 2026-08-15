#[test]
fn test_lcm_coprime() {
    assert_eq!(integer_lcm::lcm(7, 13), 91);
}

#[test]
fn test_lcm_divisor() {
    assert_eq!(integer_lcm::lcm(100, 25), 100);
}

#[test]
fn test_lcm_1() {
    assert_eq!(integer_lcm::lcm(4, 6), 12);
}

#[test]
fn test_lcm_2() {
    assert_eq!(integer_lcm::lcm(12, 18), 36);
}

#[test]
fn test_lcm_large() {
    assert_eq!(integer_lcm::lcm(21, 110), 2310);
}

#[test]
fn test_lcm_zero() {
    assert_eq!(integer_lcm::lcm(0, 5), 0);
    assert_eq!(integer_lcm::lcm(5, 0), 0);
    assert_eq!(integer_lcm::lcm(0, 0), 0);
}
