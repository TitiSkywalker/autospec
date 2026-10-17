#[test]
fn test_prime() {
    let f = integer_factors::divisors(59);
    assert_eq!(f.len(), 2);
    assert_eq!(f[0], 1);
    assert_eq!(f[1], 59);

    let f = integer_factors::divisors(1);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0], 1);

    let f = integer_factors::divisors(2);
    assert_eq!(f.len(), 2);
    assert_eq!(f[0], 1);
    assert_eq!(f[1], 2);
}

#[test]
fn test_composites() {
    let f = integer_factors::divisors(69);
    assert_eq!(f.len(), 4);
    assert_eq!(f[0], 1);
    assert_eq!(f[1], 3);
    assert_eq!(f[2], 23);
    assert_eq!(f[3], 69);

    let f = integer_factors::divisors(6);
    assert_eq!(f.len(), 4);
    assert_eq!(f[0], 1);
    assert_eq!(f[1], 2);
    assert_eq!(f[2], 3);
    assert_eq!(f[3], 6);
}

#[test]
fn test_highly_composite() {
    let f = integer_factors::divisors(360);
    assert_eq!(f.len(), 24);
    assert_eq!(f[0], 1);
    assert_eq!(f[6], 8);
    assert_eq!(f[11], 18);
    assert_eq!(f[23], 360);
}
