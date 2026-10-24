//! Minimum palindrome partitioning — dynamic programming.
//!
//! Characters are encoded as `usize` integers, exactly as in the other string
//! testcases (`string_suffix_array`, `string_aho_corasick`, ...), so no `as`
//! casts are needed.

/// Returns the minimum number of palindromic substrings into which `s` can be
/// partitioned (each piece non-empty and contiguous, and the pieces cover the
/// whole of `s` in order).
///
/// Let `dp[i]` be that minimum for the prefix `s[0..i]`.  Then `dp[0] = 0` and,
/// for `i >= 1`,
///
/// ```text
/// dp[i] = min { dp[j] + 1 : 0 <= j < i  and  s[j..i] is a palindrome }.
/// ```
///
/// The empty input has the empty partition, so the answer is `0`.
pub fn min_palindrome_partition(s: &[usize]) -> usize {
    let n = s.len();
    let mut dp = vec![0usize; n + 1];
    let mut i = 1;
    while i <= n {
        // The one-character piece `s[i-1..i]` is always a palindrome, so it
        // supplies a finite initial upper bound.
        let mut best: usize = dp[i - 1] + 1;
        let mut j = 0;
        while j < i {
            // Two inward pointers decide whether `s[j..i]` is a palindrome:
            // count the mismatched mirrored pairs.
            let mut a = j;
            let mut b = i - 1;
            let mut mismatches: usize = 0;
            while a < b {
                if s[a] != s[b] {
                    mismatches += 1;
                }
                a += 1;
                b -= 1;
            }
            if mismatches == 0 {
                let cand: usize = dp[j] + 1;
                if cand < best {
                    best = cand;
                }
            }
            j += 1;
        }
        dp[i] = best;
        i += 1;
    }
    dp[n]
}
