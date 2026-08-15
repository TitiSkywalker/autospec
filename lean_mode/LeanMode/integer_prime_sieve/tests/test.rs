#[test]
fn test_small() {
    assert_eq!(integer_prime_sieve::prime_sieve(0), vec![]);
    assert_eq!(integer_prime_sieve::prime_sieve(1), vec![]);
    assert_eq!(integer_prime_sieve::prime_sieve(2), vec![2]);
    assert_eq!(integer_prime_sieve::prime_sieve(3), vec![2, 3]);
    assert_eq!(integer_prime_sieve::prime_sieve(10), vec![2, 3, 5, 7]);
}

#[test]
fn test_medium() {
    assert_eq!(integer_prime_sieve::prime_sieve(20), vec![2, 3, 5, 7, 11, 13, 17, 19]);
    assert_eq!(
        integer_prime_sieve::prime_sieve(50),
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
    );
    assert_eq!(integer_prime_sieve::prime_sieve(100).len(), 25);
}
