//! Triangular linear-system solvers (forward / back substitution).
//!
//! Extracted from nalgebra's `solve_lower_triangular` /
//! `solve_upper_triangular` (the `SquareMatrix` triangular-solve methods) and
//! refined for Aeneas compatibility (while loops, index-based access, no
//! iterators/closures).
//!
//! Solves `L · x = b` (lower-triangular L) and `U · x = b` (upper-triangular U)
//! for an n-vector x.
//!
//! All matrices are stored in **column-major** order: element (row, col) is at
//! index `row + col * n`.  Only the diagonal and the lower (resp. upper) triangle
//! of the coefficient matrix are read.

/// Solve the linear system `L · x = b` where `L` is lower-triangular (n×n,
/// column-major) and `b` is an n-vector, via forward substitution.
///
/// * `matrix` — flattened n×n lower-triangular matrix (only the lower triangle
///   and diagonal are read)
/// * `b` — right-hand side vector (length n)
/// * `n` — dimension
///
/// # Panics
///
/// Panics if a diagonal entry is zero (the matrix is singular).  An `ok` result
/// therefore implies the diagonal is non-zero.
pub fn solve_lower_triangular(matrix: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }

    let mut x = b.to_vec();

    let mut i: usize = 0;
    while i < n {
        let diag = matrix[i + i * n];
        assert!(diag != 0.0, "singular lower-triangular matrix");

        let coeff = x[i] / diag;
        x[i] = coeff;

        // x[r] -= x[i] · L[r, i]  for r = i+1..n.
        let mut r: usize = i + 1;
        while r < n {
            let mval = matrix[r + i * n];
            let new_val = x[r] - coeff * mval;
            x[r] = new_val;
            r += 1;
        }

        i += 1;
    }

    x
}

/// Solve the linear system `U · x = b` where `U` is upper-triangular (n×n,
/// column-major) and `b` is an n-vector, via back substitution.
///
/// * `matrix` — flattened n×n upper-triangular matrix (only the upper triangle
///   and diagonal are read)
/// * `b` — right-hand side vector (length n)
/// * `n` — dimension
///
/// # Panics
///
/// Panics if a diagonal entry is zero (the matrix is singular).  An `ok` result
/// therefore implies the diagonal is non-zero.
pub fn solve_upper_triangular(matrix: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }

    let mut x = b.to_vec();

    let mut ii: usize = n;
    while ii > 0 {
        ii -= 1;
        let i = ii;

        let diag = matrix[i + i * n];
        assert!(diag != 0.0, "singular upper-triangular matrix");

        let coeff = x[i] / diag;
        x[i] = coeff;

        // x[r] -= x[i] · U[r, i]  for r = 0..i.
        let mut r: usize = 0;
        while r < i {
            let mval = matrix[r + i * n];
            let new_val = x[r] - coeff * mval;
            x[r] = new_val;
            r += 1;
        }
    }

    x
}
