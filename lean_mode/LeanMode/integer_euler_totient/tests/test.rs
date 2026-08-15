#[test]
fn test_totient_one() {
    assert_eq!(integer_euler_totient::euler_totient(1), 1);
}

#[test]
fn test_totient_prime() {
    assert_eq!(integer_euler_totient::euler_totient(2), 1);
    assert_eq!(integer_euler_totient::euler_totient(7), 6);
    assert_eq!(integer_euler_totient::euler_totient(13), 12);
}

#[test]
fn test_totient_composite() {
    assert_eq!(integer_euler_totient::euler_totient(10), 4);
    assert_eq!(integer_euler_totient::euler_totient(12), 4);
    assert_eq!(integer_euler_totient::euler_totient(15), 8);
}

#[test]
fn test_totient_large() {
    assert_eq!(integer_euler_totient::euler_totient(100), 40);
    assert_eq!(integer_euler_totient::euler_totient(1000), 400);
}
