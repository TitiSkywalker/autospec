#[test]
fn test_basic() {
    assert_eq!(integer_binomial::binomial(0, 0), 1);
    assert_eq!(integer_binomial::binomial(5, 2), 10);
    assert_eq!(integer_binomial::binomial(10, 5), 252);
    assert_eq!(integer_binomial::binomial(7, 4), 35);
    assert_eq!(integer_binomial::binomial(12, 3), 220);
}

#[test]
fn test_large() {
    assert_eq!(integer_binomial::binomial(1000, 2), 499500);
    assert_eq!(integer_binomial::binomial(20, 10), 184756);
}

#[test]
fn test_k_greater_than_n() {
    assert_eq!(integer_binomial::binomial(3, 5), 0);
}
