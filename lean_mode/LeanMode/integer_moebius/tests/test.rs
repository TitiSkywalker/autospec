#[test]
fn test_moebius_one() {
    assert_eq!(integer_moebius::moebius(1), 1);
}

#[test]
fn test_moebius_prime() {
    assert_eq!(integer_moebius::moebius(2), -1);
    assert_eq!(integer_moebius::moebius(3), -1);
    assert_eq!(integer_moebius::moebius(5), -1);
}

#[test]
fn test_moebius_square() {
    assert_eq!(integer_moebius::moebius(4), 0);
    assert_eq!(integer_moebius::moebius(9), 0);
    assert_eq!(integer_moebius::moebius(12), 0);
}

#[test]
fn test_moebius_two_factors() {
    assert_eq!(integer_moebius::moebius(6), 1);
    assert_eq!(integer_moebius::moebius(10), 1);
    assert_eq!(integer_moebius::moebius(15), 1);
}

#[test]
fn test_moebius_three_factors() {
    assert_eq!(integer_moebius::moebius(30), -1);
    assert_eq!(integer_moebius::moebius(42), -1);
}
