//! House Robber — dynamic programming solution.
//!
//! Given a list of house values, maximize the total loot without robbing
//! adjacent houses.

/// Returns the maximum total loot obtainable without robbing adjacent houses.
///
/// # Arguments
/// * `nums` — value of each house
///
/// # Returns
/// Maximum loot (0 for an empty list).
pub fn rob(nums: &[usize]) -> usize {
    let n = nums.len();
    let mut dp = vec![0usize; n + 2];

    let mut i = 0;
    while i < n {
        let take = dp[i] + nums[i];
        let skip = dp[i + 1];
        let val = if take > skip { take } else { skip };
        let idx = i + 2;
        dp[idx] = val;
        i += 1;
    }
    let result_idx = n + 1;
    dp[result_idx]
}
