//! Maximum Subarray Sum (Kadane's algorithm).
//!
//! Given a non-empty sequence of integers, find the maximum sum of any
//! contiguous subarray.

/// Returns the maximum sum of any contiguous subarray of `nums`.
///
/// Asserts that `nums` is non-empty.
///
/// # Arguments
/// * `nums` — the integer sequence
///
/// # Returns
/// Maximum contiguous subarray sum.
pub fn max_subarray_sum(nums: &[i64]) -> i64 {
    let n = nums.len();
    assert!(n > 0);
    let mut best = nums[0];
    let mut cur = nums[0];
    let mut i = 1;
    while i < n {
        let x = nums[i];
        let added = cur + x;
        cur = if added > x { added } else { x };
        if cur > best {
            best = cur;
        }
        i += 1;
    }
    best
}
