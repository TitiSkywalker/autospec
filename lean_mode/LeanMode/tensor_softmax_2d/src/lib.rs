//! 2D row-wise softmax of f64 slices (row-major storage).

/// Compute the row-wise softmax of `input`.
///
/// Input is an H×W matrix stored in row-major order.
/// For each row r: softmax2d(input)[r][j] = exp(input[r][j]) / Σₖ exp(input[r][k])
///
/// Returns an empty vector if any dimension is zero.
pub fn softmax2d(input: &[f64], rows: usize, cols: usize) -> Vec<f64> {
    if input.is_empty() || rows == 0 || cols == 0 {
        return vec![];
    }

    let mut out = vec![0.0; input.len()];
    for r in 0..rows {
        let row_start = r * cols;
        let row_end = row_start + cols;

        // Compute exp(x) and sum for this row.
        let mut sum = 0.0;
        for c in row_start..row_end {
            let e = input[c].exp();
            sum += e;
            out[c] = e;
        }

        // Normalize this row.
        for c in row_start..row_end {
            out[c] = out[c] / sum;
        }
    }
    out
}
