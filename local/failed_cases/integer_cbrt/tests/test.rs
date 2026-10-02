#[test]
fn test_small() {
    assert_eq!(integer_cbrt::icbrt(0), 0);
    assert_eq!(integer_cbrt::icbrt(1), 1);
    assert_eq!(integer_cbrt::icbrt(2), 1);
    assert_eq!(integer_cbrt::icbrt(7), 1);
    assert_eq!(integer_cbrt::icbrt(8), 2);
    assert_eq!(integer_cbrt::icbrt(9), 2);
}

#[test]
fn test_cubes() {
    assert_eq!(integer_cbrt::icbrt(26), 2);
    assert_eq!(integer_cbrt::icbrt(27), 3);
    assert_eq!(integer_cbrt::icbrt(28), 3);
    assert_eq!(integer_cbrt::icbrt(999), 9);
    assert_eq!(integer_cbrt::icbrt(1000), 10);
    assert_eq!(integer_cbrt::icbrt(1001), 10);
}

#[test]
fn test_larger() {
    assert_eq!(integer_cbrt::icbrt(1000000), 100);
    assert_eq!(integer_cbrt::icbrt(1000000000), 1000);
}
