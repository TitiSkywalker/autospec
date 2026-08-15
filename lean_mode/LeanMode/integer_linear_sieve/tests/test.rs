#[test]
fn test_primes_list() {
    let (primes, _) = integer_linear_sieve::linear_sieve(25);
    assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23]);
}

#[test]
fn test_min_prime_factor() {
    let (_, mpf) = integer_linear_sieve::linear_sieve(20);
    // mpf[k] = least prime factor of k (mpf[0]=mpf[1]=0).
    assert_eq!(mpf[2], 2);
    assert_eq!(mpf[3], 3);
    assert_eq!(mpf[4], 2);
    assert_eq!(mpf[5], 5);
    assert_eq!(mpf[6], 2);
    assert_eq!(mpf[12], 2);
    assert_eq!(mpf[14], 2);
    assert_eq!(mpf[15], 3);
    assert_eq!(mpf[17], 17);
    assert_eq!(mpf[18], 2);
    assert_eq!(mpf[20], 2);
}

#[test]
fn test_count() {
    let (primes, _) = integer_linear_sieve::linear_sieve(100);
    assert_eq!(primes.len(), 25);
}
