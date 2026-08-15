#[test]
fn test_trivial() {
    let f = integer_prime_factors::prime_factors(1);
    assert_eq!(f.len(), 0);

    let f = integer_prime_factors::prime_factors(11);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0], 11);

    let f = integer_prime_factors::prime_factors(25);
    assert_eq!(f.len(), 2);
    assert_eq!(f[0], 5);
    assert_eq!(f[1], 5);
}

#[test]
fn test_composites() {
    let f = integer_prime_factors::prime_factors(33);
    assert_eq!(f.len(), 2);
    assert_eq!(f[0], 3);
    assert_eq!(f[1], 11);

    let f = integer_prime_factors::prime_factors(2560);
    assert_eq!(f.len(), 10);
    assert_eq!(f[0], 2);
    assert_eq!(f[9], 5);

    let f = integer_prime_factors::prime_factors(360);
    assert_eq!(f.len(), 6);
    assert_eq!(f[0], 2);
    assert_eq!(f[1], 2);
    assert_eq!(f[2], 2);
    assert_eq!(f[3], 3);
    assert_eq!(f[4], 3);
    assert_eq!(f[5], 5);

    let f = integer_prime_factors::prime_factors(97);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0], 97);
}
