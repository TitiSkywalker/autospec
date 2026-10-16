//! Minimum path sum in a grid — dynamic programming.
//!
//! Given a grid of integers, find the minimum sum of a path from the top-left
//! corner to the bottom-right corner, moving only right or down at each step.

/// Returns the minimum path sum from `(0, 0)` to `(rows-1, cols-1)`.
///
/// `grid` is a flat row-major `rows × cols` matrix. Asserts that the grid is
/// non-empty (`rows > 0` and `cols > 0`).
///
/// # Arguments
/// * `grid` — flat row-major grid of integers
/// * `rows` — number of rows
/// * `cols` — number of columns
///
/// # Returns
/// Minimum sum of a top-left to bottom-right path.
pub fn min_path_sum(grid: &[i32], rows: usize, cols: usize) -> i32 {
    assert!(rows > 0);
    assert!(cols > 0);

    let dp_size = rows * cols;
    let mut dp = vec![0i32; dp_size];
    let g00 = grid[0];
    dp[0] = g00;

    // First row: only reachable from the left.
    let mut j = 1;
    while j < cols {
        let prev = dp[j - 1];
        let g = grid[j];
        let v = prev + g;
        dp[j] = v;
        j += 1;
    }

    // First column: only reachable from above.
    let mut i = 1;
    while i < rows {
        let idx = i * cols;
        let above = dp[idx - cols];
        let g = grid[idx];
        let v = above + g;
        dp[idx] = v;
        i += 1;
    }

    // Interior: take the cheaper of the left/above predecessor.
    let mut r = 1;
    while r < rows {
        let mut c = 1;
        while c < cols {
            let idx = r * cols + c;
            let above = dp[idx - cols];
            let left = dp[idx - 1];
            let best = if above < left { above } else { left };
            let g = grid[idx];
            let v = best + g;
            dp[idx] = v;
            c += 1;
        }
        r += 1;
    }

    let last = (rows - 1) * cols + (cols - 1);
    dp[last]
}
