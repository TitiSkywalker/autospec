#[test]
fn test_zero() {
    assert_eq!(integer_sqrt::isqrt(0), 0);
}

#[test]
fn test_one() {
    assert_eq!(integer_sqrt::isqrt(1), 1);
}

#[test]
fn test_small() {
    assert_eq!(integer_sqrt::isqrt(2), 1);
    assert_eq!(integer_sqrt::isqrt(3), 1);
    assert_eq!(integer_sqrt::isqrt(4), 2);
    assert_eq!(integer_sqrt::isqrt(8), 2);
    assert_eq!(integer_sqrt::isqrt(9), 3);
    assert_eq!(integer_sqrt::isqrt(15), 3);
    assert_eq!(integer_sqrt::isqrt(16), 4);
}

#[test]
fn test_larger() {
    assert_eq!(integer_sqrt::isqrt(99), 9);
    assert_eq!(integer_sqrt::isqrt(100), 10);
    assert_eq!(integer_sqrt::isqrt(101), 10);
    assert_eq!(integer_sqrt::isqrt(9999), 99);
    assert_eq!(integer_sqrt::isqrt(10000), 100);
}

#[test]
fn test_perfect_square() {
    assert_eq!(integer_sqrt::isqrt(65536), 256);
    assert_eq!(integer_sqrt::isqrt(65535), 255);
}
