//! Edit (Levenshtein) distance — dynamic programming.
//!
//! Given two sequences of integer-encoded characters, compute the minimum
//! number of single-character insertions, deletions, or substitutions needed
//! to transform one into the other.

/// Returns the Levenshtein edit distance between `a` and `b`.
///
/// # Arguments
/// * `a` — first sequence (each element is a character code)
/// * `b` — second sequence
///
/// # Returns
/// Minimum number of edit operations to turn `a` into `b`.
pub fn edit_distance(a: &[usize], b: &[usize]) -> usize {
    let n = a.len();
    let m = b.len();
    let cols = m + 1;
    let dp_size = (n + 1) * cols;
    let mut dp = vec![0usize; dp_size];

    // Base cases: dp[i][0] = i.
    let mut i = 0;
    while i <= n {
        let idx = i * cols;
        dp[idx] = i;
        i += 1;
    }
    // Base cases: dp[0][j] = j (dp[0] is already 0).
    let mut j = 1;
    while j <= m {
        dp[j] = j;
        j += 1;
    }

    // Fill the table.
    let mut r = 1;
    while r <= n {
        let mut c = 1;
        while c <= m {
            let idx = r * cols + c;
            let cost = if a[r - 1] == b[c - 1] { 0 } else { 1 };
            let above = dp[(r - 1) * cols + c];
            let left = dp[r * cols + (c - 1)];
            let diag = dp[(r - 1) * cols + (c - 1)];
            let ins = above + 1;
            let del = left + 1;
            let sub = diag + cost;
            let best = if del < sub { del } else { sub };
            let best2 = if ins < best { ins } else { best };
            dp[idx] = best2;
            c += 1;
        }
        r += 1;
    }
    dp[n * cols + m]
}
