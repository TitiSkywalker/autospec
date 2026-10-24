//! Optimal binary search tree — interval dynamic programming.
//!
//! Given the access frequencies `freq[i]` of the keys `0, 1, …, n-1`, build a
//! binary search tree minimising the expected number of nodes visited by a
//! successful search.  A key stored at depth `d` (the root has depth `0`) costs
//! `d + 1` node visits, so the quantity being minimised is
//!
//! ```text
//!     Σ_k freq[k] * (depth(k) + 1).
//! ```
//!
//! `dp[i][j]` is that minimum for the sub-range of keys `i..j`.  With the
//! convention that an empty range costs `0`,
//!
//! ```text
//!     dp[i][j] = (Σ_{t=i..j} freq[t]) + min_{i ≤ r ≤ j} (dp[i][r-1] + dp[r+1][j]),
//! ```
//!
//! because choosing `r` as the root adds one extra visit to *every* key of the
//! range (the `Σ freq` term) on top of the two independent sub-trees.

/// Minimum expected search cost of a binary search tree on the keys
/// `0, …, freq.len() - 1` with access frequencies `freq`.
pub fn optimal_bst(freq: &[usize]) -> usize {
    let n = freq.len();
    if n == 0 {
        return 0;
    }

    // Prefix sums: `pre[i] = freq[0] + … + freq[i-1]`, `pre.len() == n + 1`.
    let mut pre = vec![0usize; n + 1];
    let mut i = 0;
    while i < n {
        let s = pre[i] + freq[i];
        let ip1 = i + 1;
        pre[ip1] = s;
        i += 1;
    }

    // `dp` is a padded `(n+1) × (n+1)` matrix stored row-major; the entry for the
    // range `i..j` lives at `i * (n+1) + (j+1)`.  Cells for an empty range
    // (`j = i - 1`, i.e. column `i`) and for out-of-range `j+1 = n` are never
    // written, so they stay `0` — exactly the "empty range costs 0" convention.
    let np1 = n + 1;
    let size = np1 * np1;
    let mut dp = vec![0usize; size];

    // `len` is the number of keys in the current range.
    let mut len = 1;
    while len <= n {
        let mut i = 0;
        while i + len <= n {
            let j = i + len - 1;
            let jp1 = j + 1;
            let total = pre[jp1] - pre[i];

            // Root `r = i`: empty left range (`dp[i][i-1] = 0`), right `dp[i+1][j]`.
            let l0 = i * np1 + i;
            let r0 = (i + 1) * np1 + jp1;
            let mut best = dp[l0] + dp[r0];

            // Roots `r = i+1, …, j`.
            let mut r = i + 1;
            while r <= j {
                let li = i * np1 + r;
                let rp1 = r + 1;
                let ri = rp1 * np1 + jp1;
                let cost = dp[li] + dp[ri];
                if cost < best {
                    best = cost;
                }
                r += 1;
            }

            let val = total + best;
            let flat = i * np1 + jp1;
            dp[flat] = val;
            i += 1;
        }
        len += 1;
    }

    // The full range `0..n-1` lives at `0 * (n+1) + ((n-1)+1) = n`.
    let full = n;
    dp[full]
}
