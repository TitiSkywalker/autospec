//! Minimum-weight triangulation of a convex polygon — interval dynamic
//! programming.
//!
//! The polygon has vertices `0, 1, …, n-1` in convex position.  `weight` is a
//! flat row-major `n × n` matrix; for `i < j`, `weight[i * n + j]` is the weight
//! of the segment (side or diagonal) joining vertices `i` and `j`.  A
//! triangulation of the polygon uses exactly `n - 3` pairwise non-crossing
//! diagonals and cuts it into `n - 2` triangles.
//!
//! The cost of a triangulation is the sum, over all its triangles, of the three
//! edge weights of that triangle.  Because every polygon *side* belongs to
//! exactly one triangle and every *diagonal* belongs to exactly two, this is
//! equivalently `(sum of side weights) + 2 · (sum of diagonal weights)`.

/// Minimum-weight triangulation of the convex `n`-gon described by the flat
/// row-major `n × n` weight matrix `weight` (length `n * n`).
///
/// The interval DP uses `dp[i * n + j]` for the sub-polygon on vertices
/// `i, i+1, …, j` (with `i < j`): the base case `j = i + 1` (an edge) costs `0`,
/// and
/// `dp[i][j] = min over i < k < j of dp[i][k] + dp[k][j] + w(i,k) + w(k,j) + w(i,j)`
/// where the last three terms are the three edges of the triangle `(i, k, j)`.
/// For `n < 3` there is no triangle, so the answer is `0`.
pub fn min_triangulation(n: usize, weight: &[usize]) -> usize {
    let mut dp = vec![0usize; n * n];
    let mut len = 2;
    while len < n {
        let mut i = 0;
        while i + len < n {
            let j = i + len;
            let idx = i * n + j;
            let wij = weight[i * n + j];

            // First split k = i + 1 (always valid because j >= i + 2).
            let mut k = i + 1;
            let w1 = weight[i * n + k];
            let w2 = weight[k * n + j];
            let tri = w1 + w2 + wij;
            let left = dp[i * n + k];
            let right = dp[k * n + j];
            let mut best = left + right + tri;
            k += 1;

            // Remaining splits k = i + 2 .. j - 1.
            while k < j {
                let a1 = weight[i * n + k];
                let a2 = weight[k * n + j];
                let atri = a1 + a2 + wij;
                let l2 = dp[i * n + k];
                let r2 = dp[k * n + j];
                let cand = l2 + r2 + atri;
                if cand < best {
                    best = cand;
                }
                k += 1;
            }

            dp[idx] = best;
            i += 1;
        }
        len += 1;
    }

    if n < 3 {
        0
    } else {
        // dp[0 * n + (n - 1)] == dp[n - 1].
        dp[n - 1]
    }
}
