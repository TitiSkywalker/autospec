//! Simple f64 matrix multiplication (column-major storage).

/// Multiply two column-major matrices, returning a new matrix.
pub fn matmul(a: &[f64], b: &[f64], m: usize, k: usize, n: usize) -> Vec<f64> {
    let mut c = vec![0.0; m * n];
    for col_b in 0..n {
        for row in 0..m {
            for col_a in 0..k {
                c[row + col_b * m] += a[row + col_a * m] * b[col_a + col_b * k];
            }
        }
    }
    c
}
