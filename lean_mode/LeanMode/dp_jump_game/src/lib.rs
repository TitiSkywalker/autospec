//! Jump Game — greedy reachability.
//!
//! Given an array where each element is the maximum jump length from that
//! position, determine whether the last index is reachable from index 0.

/// Returns `true` iff the last index is reachable from index 0.
///
/// Asserts that `nums` is non-empty.
///
/// # Arguments
/// * `nums` — maximum jump length at each position
///
/// # Returns
/// `true` if the last index is reachable, `false` otherwise.
pub fn can_jump(nums: &[usize]) -> bool {
    let n = nums.len();
    assert!(n > 0);
    let mut reach = 0usize;
    let mut i = 0;
    while i < n {
        if i > reach {
            break;
        }
        let r = i + nums[i];
        if r > reach {
            reach = r;
        }
        i += 1;
    }
    let last = n - 1;
    reach >= last
}
