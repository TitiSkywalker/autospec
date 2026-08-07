//! LDLᵀ decomposition for n×n symmetric matrices.
//!
//! Factorises A such that A = L · D · Lᵀ, where:
//! - L is unit lower triangular (1's on the diagonal),
//! - D is diagonal.
//!
//! No square roots are required — all operations are +, -, *, /.
//!
//! Only the lower triangle of the input is accessed (the matrix is assumed
//! symmetric).  The result is stored in a single n×n matrix (row-major):
//! * `i = j` → D[i]
//! * `i > j` → L[i][j]
//! * `i < j` → 0

/// Result of LDLᵀ decomposition.
pub struct Ldlt {
    /// Combined L and D factors (n×n, row-major).
    /// * `i = j` → `ld[i*n + i]` is D[i]
    /// * `i > j` → `ld[i*n + j]` is L[i][j]
    /// * `i < j` → `ld[i*n + j]` is 0
    pub ld: Vec<f64>,
}

/// Compute the LDLᵀ decomposition of an n×n symmetric matrix.
///
/// * `matrix` — flattened n×n matrix in row-major order (only lower triangle used)
/// * `n` — dimension
///
/// # Panics
///
/// Panics if a zero pivot is encountered (matrix is singular / not suitable
/// for LDLᵀ without pivoting).
pub fn ldlt_decomposition(matrix: &[f64], n: usize) -> Ldlt {
    if n == 0 {
        return Ldlt { ld: Vec::new() };
    }

    // Copy the lower triangle of A into the working array.
    let mut ld = matrix.to_vec();

    let mut j: usize = 0;
    while j < n {
        // ------------------------------------------------------------------
        // Subtract contributions from previous columns k < j:
        //   L[i][j]·D[j] = A[i][j] - Σ_{k<j} L[i][k]·D[k]·L[j][k]
        // ------------------------------------------------------------------
        let mut k: usize = 0;
        while k < j {
            let dk = ld[k * n + k]; // D[k]
            let ljk = ld[j * n + k]; // L[j][k]
            let factor = dk * ljk; // D[k] · L[j][k]
            let mut i: usize = j;
            while i < n {
                ld[i * n + j] = ld[i * n + j] - ld[i * n + k] * factor;
                i += 1;
            }
            k += 1;
        }

        // After the k-loop: ld[j*n + j] = D[j].
        let diag = ld[j * n + j];
        assert!(diag != 0.0, "matrix is singular");

        // Normalise below-diagonal entries: L[i][j] /= D[j]  (i > j).
        let mut i: usize = j + 1;
        while i < n {
            ld[i * n + j] = ld[i * n + j] / diag;
            i += 1;
        }

        // Zero out above-diagonal entries (clean output).
        let mut i: usize = 0;
        while i < j {
            ld[i * n + j] = 0.0;
            i += 1;
        }

        j += 1;
    }

    Ldlt { ld }
}
