//! n×n matrix inversion via Gauss-Jordan elimination with partial pivoting.
//!
//! The matrix is stored in row-major order as a flat `&[f64]`.
//! Panics if the matrix is singular.

/// Compute the inverse of an n×n matrix via Gauss-Jordan elimination.
///
/// * `matrix` — flattened n×n matrix in row-major order
/// * `n` — dimension
///
/// Returns the inverse (n×n, row-major).
///
/// # Panics
///
/// Panics if the matrix is singular (non-invertible).
/// For `n = 0` the empty matrix is its own inverse.
pub fn matrix_inverse(matrix: &[f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }

    let total_cols = 2 * n;

    // Build the augmented matrix [A | I] — n rows × 2n columns.
    let aug_size = n * total_cols;
    let mut aug: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < aug_size {
        aug.push(0.0);
        idx += 1;
    }

    // Fill the left half with the input matrix.
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            aug[i * total_cols + j] = matrix[i * n + j];
            j += 1;
        }
        i += 1;
    }

    // Fill the right half with the identity matrix.
    let mut i: usize = 0;
    while i < n {
        aug[i * total_cols + n + i] = 1.0;
        i += 1;
    }

    // ------------------------------------------------------------------
    // Gauss-Jordan elimination
    // ------------------------------------------------------------------
    let mut col: usize = 0;
    while col < n {
        // -- Partial pivoting ------------------------------------------
        let mut pivot_row = col;
        let mut max_val = aug[col * total_cols + col];
        if max_val < 0.0 {
            max_val = -max_val;
        }

        let mut row: usize = col + 1;
        while row < n {
            let mut val = aug[row * total_cols + col];
            if val < 0.0 {
                val = -val;
            }
            if val > max_val {
                max_val = val;
                pivot_row = row;
            }
            row += 1;
        }

        // Singular matrix — no inverse.
        assert!(max_val != 0.0, "singular matrix");

        // -- Swap rows (if needed) ------------------------------------
        if pivot_row != col {
            let mut j: usize = 0;
            while j < total_cols {
                let tmp = aug[col * total_cols + j];
                aug[col * total_cols + j] = aug[pivot_row * total_cols + j];
                aug[pivot_row * total_cols + j] = tmp;
                j += 1;
            }
        }

        // -- Normalize the pivot row ----------------------------------
        let pivot = aug[col * total_cols + col];
        let mut j: usize = 0;
        while j < total_cols {
            aug[col * total_cols + j] = aug[col * total_cols + j] / pivot;
            j += 1;
        }

        // -- Eliminate all other rows ---------------------------------
        let mut row: usize = 0;
        while row < n {
            if row != col {
                let factor = aug[row * total_cols + col];
                let mut j: usize = 0;
                while j < total_cols {
                    aug[row * total_cols + j] =
                        aug[row * total_cols + j] - factor * aug[col * total_cols + j];
                    j += 1;
                }
            }
            row += 1;
        }

        col += 1;
    }

    // ------------------------------------------------------------------
    // Extract the right half of the augmented matrix (the inverse).
    // ------------------------------------------------------------------
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
            result[i * n + j] = aug[i * total_cols + n + j];
            j += 1;
        }
        i += 1;
    }

    result
}
