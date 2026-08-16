//! Cholesky decomposition of a symmetric positive-definite matrix.
//!
//! Extracted from nalgebra's `Cholesky::new_unchecked` and refined for Aeneas
//! compatibility (while loops, index-based access, no iterators/closures).
//!
//! Decomposes a symmetric positive-definite n×n matrix A into A = L · Lᵀ, where
//! L is lower-triangular with a positive diagonal.
//!
//! All matrices are stored in **column-major** order: element (row, col) is at
//! index `row + col * n`.  Only the lower triangle (including the diagonal) of
//! the input is read.

/// Result of the Cholesky decomposition.
pub struct Cholesky {
    /// Lower-triangular factor L (n×n, column-major), with a positive diagonal.
    pub l: Vec<f64>,
}

/// Compute the Cholesky decomposition of a symmetric positive-definite n×n
/// matrix (column-major).
///
/// Returns L (n×n, lower-triangular) such that A = L · Lᵀ.
///
/// * `matrix` — flattened n×n matrix in column-major order (only the lower
///   triangle is read)
/// * `n` — dimension
///
/// # Panics
///
/// Panics if a diagonal pivot is not strictly positive, i.e. the (lower
/// triangle of the) matrix is not positive-definite.  An `ok` result therefore
/// implies the matrix was positive-definite.
pub fn cholesky_decomposition(matrix: &[f64], n: usize) -> Cholesky {
    if n == 0 {
        return Cholesky { l: Vec::new() };
    }

    // Working copy; the lower triangle becomes L in place.
    let mut chol = matrix.to_vec();

    let mut j: usize = 0;
    while j < n {
        // Subtract the contribution of the already-computed columns k < j:
        //   col_j[j..n] -= L[j, k] · col_k[j..n].
        let mut k: usize = 0;
        while k < j {
            let factor = -chol[j + k * n];
            let mut i: usize = j;
            while i < n {
                let idx = i + j * n;
                let idx_k = i + k * n;
                let prod = factor * chol[idx_k];
                let new_val = chol[idx] + prod;
                chol[idx] = new_val;
                i += 1;
            }
            k += 1;
        }

        let diag = chol[j + j * n];
        assert!(diag > 0.0, "matrix is not positive-definite");
        let denom = diag.sqrt();
        chol[j + j * n] = denom;

        // Scale the sub-diagonal of column j: col_j[j+1..n] /= L[j, j].
        let mut i: usize = j + 1;
        while i < n {
            let idx = i + j * n;
            let new_val = chol[idx] / denom;
            chol[idx] = new_val;
            i += 1;
        }

        j += 1;
    }

    // Extract the lower triangle (strict upper triangle is zeroed).
    let mut l = vec![0.0; n * n];
    let mut col: usize = 0;
    while col < n {
        let mut row: usize = col;
        while row < n {
            l[row + col * n] = chol[row + col * n];
            row += 1;
        }
        col += 1;
    }

    Cholesky { l }
}
