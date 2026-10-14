//! Two Sum via the two-pointer technique on a sorted slice.

/// Returns `true` if there are two distinct indices `i < j` with
/// `nums[i] + nums[j] == target`.
///
/// `nums` must be sorted in non-decreasing order. The two pointers start at
/// the ends and walk inward: if the sum is too small the left pointer moves
/// right, if too large the right pointer moves left.
pub fn two_sum(nums: &[i32], target: i32) -> bool {
    let n = nums.len();
    let mut l: usize = 0;
    let mut r: usize = n;
    let mut found = false;
    while l + 1 < r {
        let ri = r - 1;
        let sum = nums[l] + nums[ri];
        if sum == target {
            found = true;
            l = r;
        } else if sum < target {
            l += 1;
        } else {
            r -= 1;
        }
    }
    found
}
