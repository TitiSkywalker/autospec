#[test]
fn test_factorial_zero() {
    assert_eq!(integer_factorial::factorial(0), 1);
}

#[test]
fn test_factorial_one() {
    assert_eq!(integer_factorial::factorial(1), 1);
}

#[test]
fn test_factorial_small() {
    assert_eq!(integer_factorial::factorial(5), 120);
    assert_eq!(integer_factorial::factorial(6), 720);
}

#[test]
fn test_factorial_ten() {
    assert_eq!(integer_factorial::factorial(10), 3628800);
}
