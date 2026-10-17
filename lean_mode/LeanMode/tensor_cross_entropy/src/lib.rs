//! 2D cross-entropy loss (row-major storage).

/// Compute the cross-entropy loss between predicted probabilities `pred` and
/// target probabilities `target` (both H×W, row-major):
///
///   loss = -Σ_r Σ_c target[r][c] · log(pred[r][c])
///
/// Precondition: every `target` entry is non-negative and every `pred` entry
/// is strictly positive. Terms with `target <= 0` are skipped (the convention
/// `0 · log p = 0`), so a one-hot target paired with a matching one-hot
/// prediction yields 0.0.
///
/// Returns 0.0 if either dimension is zero.
pub fn cross_entropy(pred: &[f64], target: &[f64], rows: usize, cols: usize) -> f64 {
    if rows == 0 || cols == 0 {
        return 0.0;
    }
    let len = rows * cols;
    let mut total = 0.0;
    let mut i: usize = 0;
    while i < len {
        let t = target[i];
        if t > 0.0 {
            let lp = pred[i].ln();
            total = total - t * lp;
        }
        i = i + 1;
    }
    total
}
