//! 0/1 Knapsack Problem — dynamic programming solution.
//!
//! Given a set of items, each with a weight and a value, determine the maximum
//! total value that can be obtained without exceeding the capacity.
//! Each item can be used at most once (0/1).

/// Solves the 0/1 knapsack problem using dynamic programming.
///
/// # Arguments
/// * `weights` — item weights
/// * `values` — item values
/// * `capacity` — maximum weight capacity
///
/// # Returns
/// Maximum total value achievable without exceeding the capacity.
pub fn knapsack(weights: &[usize], values: &[usize], capacity: usize) -> usize {
    let n = weights.len();
    let ncols = capacity + 1;
    let dp_size = (n + 1) * ncols;
    let mut dp = vec![0usize; dp_size];

    let mut i = 1;
    while i <= n {
        let wi = weights[i - 1];
        let vi = values[i - 1];
        let mut w = 0;
        while w <= capacity {
            let idx = i * ncols + w;
            if wi <= w {
                let idx_include = (i - 1) * ncols + (w - wi);
                let idx_exclude = (i - 1) * ncols + w;
                let include = vi + dp[idx_include];
                let exclude = dp[idx_exclude];
                let val = if include > exclude { include } else { exclude };
                dp[idx] = val;
            } else {
                let idx_prev = (i - 1) * ncols + w;
                let prev = dp[idx_prev];
                dp[idx] = prev;
            }
            w += 1;
        }
        i += 1;
    }
    let result_idx = n * ncols + capacity;
    dp[result_idx]
}