//! LU decomposition with partial (row) pivoting for n×n matrices.
//!
//! The matrix is stored in row-major order as a flat `&[f64]`.
//! Factorises A such that P * A = L * U, where:
//! - L is lower triangular with unit diagonal,
//! - U is upper triangular,
//! - P is a permutation matrix.
//!
//! L and U share a single n×n matrix: the diagonal and above hold U,
//! the strictly lower part holds the L multipliers.

/// Result of LU decomposition.
pub struct LU {
    /// Combined L and U factors (n×n, row-major).
    /// * `i ≤ j` → `lu[i*n + j]` is U[i][j]
    /// * `i > j` → `lu[i*n + j]` is L[i][j]  (L has implicit 1's on the diagonal)
    pub lu: Vec<f64>,
    /// Permutation: `perm[i]` is the original row that now sits at row i.
    pub perm: Vec<usize>,
}

/// Compute the LU decomposition of an n×n matrix with partial pivoting.
///
/// * `matrix` — flattened n×n matrix in row-major order
/// * `n` — dimension
///
/// # Panics
///
/// Panics if the matrix is singular.
pub fn lu_decomposition(matrix: &[f64], n: usize) -> LU {
    if n == 0 {
        return LU {
            lu: Vec::new(),
            perm: Vec::new(),
        };
    }

    // Copy the input matrix into the working array.
    let mut lu = matrix.to_vec();

    // Initialise the permutation to identity.
    let mut perm: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < n {
        perm.push(i);
        i += 1;
    }

    let mut k: usize = 0;
    while k < n {
        // ------------------------------------------------------------------
        // Partial pivoting — find the row with the largest |value| in column k
        // ------------------------------------------------------------------
        let mut max_abs = lu[k * n + k];
        if max_abs < 0.0 {
            max_abs = -max_abs;
        }
        let mut piv = k;

        let mut i: usize = k + 1;
        while i < n {
            let mut abs_val = lu[i * n + k];
            if abs_val < 0.0 {
                abs_val = -abs_val;
            }
            if abs_val > max_abs {
                max_abs = abs_val;
                piv = i;
            }
            i += 1;
        }

        // Singular matrix — pivot is (near) zero.
        let diag = lu[piv * n + k];
        assert!(diag != 0.0, "singular matrix");

        // ------------------------------------------------------------------
        // Row swap (if needed)
        // ------------------------------------------------------------------
        if piv != k {
            let tmp_perm = perm[k];
            perm[k] = perm[piv];
            perm[piv] = tmp_perm;

            let mut j: usize = 0;
            while j < n {
                let tmp = lu[k * n + j];
                lu[k * n + j] = lu[piv * n + j];
                lu[piv * n + j] = tmp;
                j += 1;
            }
        }

        // ------------------------------------------------------------------
        // Gaussian elimination — eliminate rows below the pivot
        // ------------------------------------------------------------------
        let mut i: usize = k + 1;
        while i < n {
            let mult = lu[i * n + k] / diag;
            lu[i * n + k] = mult;

            let mut j: usize = k + 1;
            while j < n {
                lu[i * n + j] = lu[i * n + j] - mult * lu[k * n + j];
                j += 1;
            }
            i += 1;
        }

        k += 1;
    }

    LU { lu, perm }
}
