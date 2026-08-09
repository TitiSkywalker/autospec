//! f64 matrix addition (row-major storage).

/// Add two row-major matrices A + B, returning a new matrix.
/// A, B, and the result are all m×n.
pub fn matadd(a: &[f64], b: &[f64], m: usize, n: usize) -> Vec<f64> {
    let len = m * n;
    let mut c = vec![0.0; len];
    for i in 0..len {
        c[i] = a[i] + b[i];
    }
    c
}
