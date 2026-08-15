#[test]
fn test_small() {
    assert_eq!(integer_euler_totient_sieve::euler_totient_sieve(0), vec![]);
    assert_eq!(integer_euler_totient_sieve::euler_totient_sieve(1), vec![1]);
    assert_eq!(integer_euler_totient_sieve::euler_totient_sieve(4), vec![1, 1, 2, 2]);
    assert_eq!(integer_euler_totient_sieve::euler_totient_sieve(7), vec![1, 1, 2, 2, 4, 2, 6]);
}

#[test]
fn test_larger() {
    assert_eq!(
        integer_euler_totient_sieve::euler_totient_sieve(12),
        vec![1, 1, 2, 2, 4, 2, 6, 4, 6, 4, 10, 4]
    );
    assert_eq!(integer_euler_totient_sieve::euler_totient_sieve(20), vec![1, 1, 2, 2, 4, 2, 6, 4, 6, 4, 10, 4, 12, 6, 8, 8, 16, 6, 18, 8]);
}
