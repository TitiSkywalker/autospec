// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_basic_case() {
    let coins = vec![1, 2, 5];
    let result = dp_coin_change::coin_change(&coins, 11);
    assert_eq!(result, 3);
}

#[test]
fn test_multiple_denominations() {
    let coins = vec![2, 3, 5, 7, 11];
    let result = dp_coin_change::coin_change(&coins, 119);
    assert_eq!(result, 12);
}

#[test]
fn test_exact_combination() {
    let coins = vec![1, 3, 4];
    let result = dp_coin_change::coin_change(&coins, 6);
    assert_eq!(result, 2);
}

#[test]
fn test_greedy_fails() {
    let coins = vec![1, 12, 20];
    let result = dp_coin_change::coin_change(&coins, 24);
    assert_eq!(result, 2);
}

#[test]
fn test_single_coin_large_amount() {
    let coins = vec![1];
    let result = dp_coin_change::coin_change(&coins, 100);
    assert_eq!(result, 100);
}

#[test]
fn test_large_amount_multiple_coins() {
    let coins = vec![1, 2, 5];
    let result = dp_coin_change::coin_change(&coins, 10000);
    assert_eq!(result, 2000);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn test_zero_amount() {
    let coins = vec![1, 2, 3];
    let result = dp_coin_change::coin_change(&coins, 0);
    assert_eq!(result, 0);
}

#[test]
fn test_non_standard_denominations() {
    let coins = vec![1, 4, 6, 9];
    let result = dp_coin_change::coin_change(&coins, 15);
    assert_eq!(result, 2);
}

#[test]
fn test_prime_coins() {
    let coins = vec![2, 3, 5, 7];
    let result = dp_coin_change::coin_change(&coins, 17);
    assert_eq!(result, 3);
}

// ── Property tests ──────────────────────────────────────────────────────────

#[test]
fn test_powers_of_two() {
    let coins = vec![1, 2, 4, 8, 16, 32, 64];
    let result = dp_coin_change::coin_change(&coins, 127);
    assert_eq!(result, 7);
}

#[test]
fn test_fibonacci() {
    let coins = vec![1, 2, 3, 5, 8, 13, 21, 34];
    let result = dp_coin_change::coin_change(&coins, 55);
    assert_eq!(result, 2);
}

// ── Failure tests (assert! panics → #[should_panic]) ────────────────────────

#[test]
#[should_panic]
fn test_empty_coins_panics() {
    let coins: Vec<usize> = vec![];
    dp_coin_change::coin_change(&coins, 1);
}

#[test]
#[should_panic]
fn test_no_solution_panics() {
    let coins = vec![2];
    dp_coin_change::coin_change(&coins, 3);
}

#[test]
#[should_panic]
fn test_coins_larger_than_amount_panics() {
    let coins = vec![5, 10, 20];
    dp_coin_change::coin_change(&coins, 1);
}