//! f64 matrix multiplication (row-major storage).

/// Multiply two row-major matrices, returning a new matrix.
/// A: m×k, B: k×n, result: m×n
pub fn matmul(a: &[f64], b: &[f64], m: usize, k: usize, n: usize) -> Vec<f64> {
    let mut c = vec![0.0; m * n];
    for row in 0..m {
        for col in 0..n {
            for t in 0..k {
                c[row * n + col] += a[row * k + t] * b[t * n + col];
            }
        }
    }
    c
}
