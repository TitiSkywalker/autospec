//! n×n matrix transpose — `A^T[i][j] = A[j][i]`.
//!
//! The matrix is stored in row-major order as a flat `&[f64]`.

/// Compute the transpose of an n×n matrix.
///
/// * `matrix` — flattened n×n matrix in row-major order
/// * `n` — dimension
///
/// Returns the transpose (n×n, row-major).
/// For `n = 0` returns the empty matrix.
pub fn transpose(matrix: &[f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }

    let mut result: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < n * n {
        result.push(0.0);
        idx += 1;
    }

    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            result[i * n + j] = matrix[j * n + i];
            j += 1;
        }
        i += 1;
    }

    result
}
