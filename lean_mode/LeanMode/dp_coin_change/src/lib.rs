//! Coin Change Problem — dynamic programming solution.
//!
//! Given a set of coin denominations and a target amount, find the minimum
//! number of coins needed to make up the amount. Each coin can be used
//! unlimited times. Asserts that the amount can be made up with the given coins.

/// Returns the minimum number of coins needed to make up `amount`.
///
/// Asserts that a valid combination exists; panics if no combination works.
///
/// # Arguments
/// * `coins` — available coin denominations
/// * `amount` — target amount
///
/// # Returns
/// Minimum number of coins required.
pub fn coin_change(coins: &[usize], amount: usize) -> usize {
    let sentinel = amount + 1;
    let dp_size = amount + 1;
    let mut dp = vec![sentinel; dp_size];
    dp[0] = 0;

    let mut i = 1;
    while i <= amount {
        let mut best = sentinel;
        let mut j = 0;
        while j < coins.len() {
            let coin = coins[j];
            if coin <= i {
                let prev = dp[i - coin];
                let candidate = prev + 1;
                if candidate < best {
                    best = candidate;
                }
            }
            j += 1;
        }
        dp[i] = best;
        i += 1;
    }

    let result = dp[amount];
    assert!(result != sentinel);
    result
}