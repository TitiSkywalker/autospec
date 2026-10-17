/// Test: basic known-answer — gcd(48, 18) = 6
#[test]
fn test_known_answer_basic() {
    let result = integer_gcd::compute_gcd(48, 18);
    assert_eq!(result, 6);
}

/// Test: coprime numbers — gcd(17, 13) = 1
#[test]
fn test_coprime() {
    let result = integer_gcd::compute_gcd(17, 13);
    assert_eq!(result, 1);
}

/// Test: edge case — both zero — gcd(0, 0) = 0
#[test]
fn test_edge_case_both_zero() {
    let result = integer_gcd::compute_gcd(0, 0);
    assert_eq!(result, 0);
}

/// Test: edge case — first argument zero — gcd(0, y) = y
#[test]
fn test_edge_case_first_zero() {
    let result = integer_gcd::compute_gcd(0, 42);
    assert_eq!(result, 42);
}

/// Test: edge case — second argument zero — gcd(x, 0) = x
#[test]
fn test_edge_case_second_zero() {
    let result = integer_gcd::compute_gcd(100, 0);
    assert_eq!(result, 100);
}

/// Test: edge case — same numbers — gcd(42, 42) = 42
#[test]
fn test_edge_case_same_numbers() {
    let result = integer_gcd::compute_gcd(42, 42);
    assert_eq!(result, 42);
}

/// Test: one argument is a multiple of the other — gcd(100, 25) = 25
#[test]
fn test_one_is_multiple() {
    let result = integer_gcd::compute_gcd(100, 25);
    assert_eq!(result, 25);
}

/// Test: larger numbers — gcd(1071, 462) = 21
#[test]
fn test_larger_numbers() {
    let result = integer_gcd::compute_gcd(1071, 462);
    assert_eq!(result, 21);
}

/// Test: large prime arguments — gcd(7919, 7907) = 1
#[test]
fn test_large_primes() {
    let result = integer_gcd::compute_gcd(7919, 7907);
    assert_eq!(result, 1);
}
