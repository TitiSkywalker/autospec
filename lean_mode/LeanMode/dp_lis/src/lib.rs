//! Longest Increasing Subsequence (LIS) — O(n²) dynamic programming.
//!
//! Given a sequence of integers, find the length of the longest strictly
//! increasing subsequence.

/// Returns the length of the longest strictly increasing subsequence of `nums`.
///
/// # Arguments
/// * `nums` — the integer sequence
///
/// # Returns
/// Length of the longest strictly increasing subsequence.
pub fn lis(nums: &[i32]) -> usize {
    let n = nums.len();
    let mut dp = vec![1usize; n];
    let mut best = 0usize;

    let mut i = 0;
    while i < n {
        let mut max_prev = 0usize;
        let mut j = 0;
        while j < i {
            if nums[j] < nums[i] {
                let cand = dp[j];
                if cand > max_prev {
                    max_prev = cand;
                }
            }
            j += 1;
        }
        let val = max_prev + 1;
        dp[i] = val;
        if val > best {
            best = val;
        }
        i += 1;
    }
    best
}
