//! 2D sigmoid activation (element-wise, row-major storage).

/// Apply the logistic sigmoid to every element of an H×W matrix (row-major).
///
///   σ(x) = 1 / (1 + exp(-x))
///
/// Returns an empty vector if either dimension is zero.
pub fn sigmoid2d(input: &[f64], rows: usize, cols: usize) -> Vec<f64> {
    if rows == 0 || cols == 0 {
        return vec![];
    }
    let len = rows * cols;
    let mut out = vec![0.0; len];
    let mut i: usize = 0;
    while i < len {
        let e = (-input[i]).exp();
        out[i] = 1.0 / (1.0 + e);
        i = i + 1;
    }
    out
}
