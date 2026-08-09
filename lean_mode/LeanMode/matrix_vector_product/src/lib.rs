//! Matrix-vector multiplication (row-major storage).

/// Multiply row-major m×n matrix A by vector x of length n.
/// Returns vector y of length m.
pub fn matvec(a: &[f64], x: &[f64], m: usize, n: usize) -> Vec<f64> {
    let mut y = vec![0.0; m];
    for i in 0..m {
        for j in 0..n {
            y[i] += a[i * n + j] * x[j];
        }
    }
    y
}
