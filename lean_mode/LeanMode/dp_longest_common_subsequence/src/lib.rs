//! Longest Common Subsequence (LCS) — dynamic programming.
//!
//! Given two sequences, find the length of the longest subsequence
//! present in both.

/// Returns the length of the longest common subsequence of `a` and `b`.
///
/// # Arguments
/// * `a` — first sequence
/// * `b` — second sequence
///
/// # Returns
/// Length of the longest common subsequence.
pub fn lcs(a: &[usize], b: &[usize]) -> usize {
    let n = a.len();
    let m = b.len();
    let ncols = m + 1;
    let dp_size = (n + 1) * ncols;
    let mut dp = vec![0usize; dp_size];

    let mut i = 1;
    while i <= n {
        let mut j = 1;
        while j <= m {
            let idx = i * ncols + j;
            if a[i - 1] == b[j - 1] {
                let prev = dp[(i - 1) * ncols + (j - 1)];
                let val = prev + 1;
                dp[idx] = val;
            } else {
                let up = dp[(i - 1) * ncols + j];
                let left = dp[i * ncols + (j - 1)];
                let val = if up > left { up } else { left };
                dp[idx] = val;
            }
            j += 1;
        }
        i += 1;
    }
    dp[n * ncols + m]
}
