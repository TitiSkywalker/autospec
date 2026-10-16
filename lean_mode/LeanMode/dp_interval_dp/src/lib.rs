//! Matrix chain multiplication — interval dynamic programming.
//!
//! Given a chain of matrices with dimensions `dims[0] × dims[1]`,
//! `dims[1] × dims[2]`, …, `dims[n-1] × dims[n]`, find the minimum number of
//! scalar multiplications needed to compute the full product.

/// Returns the minimum number of scalar multiplications to multiply the whole
/// chain (there are `n = dims.len() - 1` matrices).
///
/// # Arguments
/// * `dims` — dimension array of length `n + 1`; matrix `i` is
///   `dims[i] × dims[i+1]`
///
/// # Returns
/// Minimum scalar-multiplication count for the full parenthesization.
pub fn matrix_chain_order(dims: &[usize]) -> usize {
    let len = dims.len();
    let n = if len == 0 { 0 } else { len - 1 };
    let dp_size = n * n;
    let mut dp = vec![0usize; dp_size];

    // Interval length `l` from 2 up to n; `i` is the left matrix index.
    let mut l = 2;
    while l <= n {
        let mut i = 0;
        while i + l <= n {
            let j = i + l - 1;
            let idx = i * n + j;
            let da = dims[i];
            let dc = dims[j + 1];

            // First split k = i.
            let mut k = i;
            let db0 = dims[k + 1];
            let left0 = dp[i * n + k];
            let right0 = dp[(k + 1) * n + j];
            let p1 = da * db0;
            let p2 = p1 * dc;
            let mut best = left0 + right0 + p2;
            k += 1;
            // Remaining splits k = i+1 .. j-1.
            while k < j {
                let db = dims[k + 1];
                let left = dp[i * n + k];
                let right = dp[(k + 1) * n + j];
                let p1 = da * db;
                let p2 = p1 * dc;
                let cost = left + right + p2;
                if cost < best {
                    best = cost;
                }
                k += 1;
            }
            dp[idx] = best;
            i += 1;
        }
        l += 1;
    }

    if n == 0 {
        0
    } else {
        dp[n - 1]
    }
}
