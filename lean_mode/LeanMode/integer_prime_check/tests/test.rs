#[test]
fn test_primes() {
    assert!(integer_prime_check::is_prime(2));
    assert!(integer_prime_check::is_prime(3));
    assert!(integer_prime_check::is_prime(5));
    assert!(integer_prime_check::is_prime(7));
    assert!(integer_prime_check::is_prime(11));
    assert!(integer_prime_check::is_prime(13));
    assert!(integer_prime_check::is_prime(97));
    assert!(integer_prime_check::is_prime(2003));
}

#[test]
fn test_composites() {
    assert!(!integer_prime_check::is_prime(0));
    assert!(!integer_prime_check::is_prime(1));
    assert!(!integer_prime_check::is_prime(4));
    assert!(!integer_prime_check::is_prime(6));
    assert!(!integer_prime_check::is_prime(9));
    assert!(!integer_prime_check::is_prime(21));
    assert!(!integer_prime_check::is_prime(25));
    assert!(!integer_prime_check::is_prime(2004));
}
