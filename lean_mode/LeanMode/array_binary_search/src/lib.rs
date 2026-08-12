//! Lower-bound binary search — returns the first index `i` such that
//! `input[i] >= target`, or `input.len()` if no such index exists.

/// Returns the first index `i` where `input[i] >= target`.
///
/// The input slice must be sorted in non-decreasing order. If all elements are strictly less than `target`, returns `input.len()`.
pub fn binary_search_lower_bound(input: &[i32], target: i32) -> usize {
    let len = input.len();
    let mut lo: usize = 0;
    let mut hi: usize = len;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if input[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}
