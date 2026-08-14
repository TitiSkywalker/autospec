//! QR decomposition using Householder reflections.
//!
//! Extracted from nalgebra's `QR::new` and refined for Aeneas compatibility
//! (while loops, index-based access, no iterators/closures).
//!
//! Decomposes an m×n matrix A into A = Q · R, where Q (m×k) has orthonormal
//! columns and R (k×n) is upper triangular, with k = min(m, n).
//!
//! All matrices are stored in **column-major** order: element (row, col) is
//! at index `row + col * nrows` (for Q: `nrows` rows; for R: `k` rows).

/// Result of the reduced QR decomposition.
pub struct Qr {
    /// Q factor: m×k matrix with orthonormal columns (column-major).
    pub q: Vec<f64>,
    /// R factor: k×n upper-triangular matrix (column-major).
    pub r: Vec<f64>,
}

/// Compute the Householder reflection axis for a column vector, in place.
///
/// Given `column[0..len]`, computes the reflection that zeros out all
/// entries except the first.  The vector is normalized to unit length.
///
/// Returns `(signed_norm, not_zero)` where `signed_norm` is the signed
/// Euclidean norm of the original vector.
fn reflection_axis(column: &mut Vec<f64>, len: usize) -> (f64, bool) {
    if len == 0 {
        return (0.0, false);
    }

    // Squared Euclidean norm of the column.
    let mut sq_norm: f64 = 0.0;
    let mut i: usize = 0;
    while i < len {
        sq_norm = sq_norm + column[i] * column[i];
        i += 1;
    }
    let norm = sq_norm.sqrt();
    if norm == 0.0 {
        return (0.0, false);
    }

    let first = column[0];
    let sign = if first < 0.0 { -1.0 } else { 1.0 };
    let modulus = if first < 0.0 { -first } else { first };
    let signed_norm = sign * norm;

    // ‖v‖² = 2·(‖x‖² + |x₀|·‖x‖) for v = x + sign(x₀)·‖x‖·e₁.
    let factor = (sq_norm + modulus * norm) * 2.0;
    if factor == 0.0 {
        return (signed_norm, false);
    }

    column[0] = first + signed_norm;

    let factor_sqrt = factor.sqrt();
    let mut i: usize = 0;
    while i < len {
        column[i] = column[i] / factor_sqrt;
        i += 1;
    }

    (-signed_norm, true)
}

/// Eliminate the sub-diagonal of column `icol` with a Householder
/// reflection and apply it to the trailing columns.
///
/// Returns the diagonal entry (the signed norm of the column vector).
fn clear_column(
    mut matrix: Vec<f64>,
    nrows: usize,
    ncols: usize,
    icol: usize,
) -> (f64, Vec<f64>) {
    let axis_start = icol * nrows + icol;
    let axis_len = nrows - icol;
    if axis_len == 0 {
        return (0.0, matrix);
    }

    // Extract the axis from column `icol`, starting at row `icol`.
    let mut axis = vec![0.0; axis_len];
    let mut i: usize = 0;
    while i < axis_len {
        axis[i] = matrix[axis_start + i];
        i += 1;
    }

    let (reflection_norm, not_zero) = reflection_axis(&mut axis, axis_len);

    if !not_zero {
        let mut i: usize = 0;
        while i < axis_len {
            matrix[axis_start + i] = axis[i];
            i += 1;
        }
        return (reflection_norm, matrix);
    }

    // sign = signum(reflection_norm); reflection_norm is non-zero here.
    let sign = if reflection_norm < 0.0 { -1.0 } else { 1.0 };
    let right_len = nrows - icol;

    let mut right_col: usize = icol + 1;
    while right_col < ncols {
        let right_col_start = right_col * nrows + icol;

        // dot = axis · column[right_col][icol..nrows]
        let mut dot: f64 = 0.0;
        let mut i: usize = 0;
        while i < right_len {
            dot = dot + axis[i] * matrix[right_col_start + i];
            i += 1;
        }

        // column := sign · column + (sign · -2 · dot) · axis
        let factor = dot * (sign * -2.0);
        let mut i: usize = 0;
        while i < right_len {
            matrix[right_col_start + i] =
                factor * axis[i] + sign * matrix[right_col_start + i];
            i += 1;
        }

        right_col += 1;
    }

    // Write the (normalized) axis back into column `icol`.
    let mut i: usize = 0;
    while i < axis_len {
        matrix[axis_start + i] = axis[i];
        i += 1;
    }

    (reflection_norm, matrix)
}

/// Reconstruct the Q factor (nrows × min_dim, column-major) by applying the
/// Householder reflections to the first min_dim columns of the identity in
/// reverse order.
fn build_q(compact: &[f64], diag: &[f64], nrows: usize, ncols: usize) -> Vec<f64> {
    let min_dim = if nrows < ncols { nrows } else { ncols };

    let mut q = vec![0.0; nrows * min_dim];
    let mut i: usize = 0;
    while i < min_dim {
        q[i + i * nrows] = 1.0;
        i += 1;
    }

    // Apply the reflections in reverse order.
    let mut ii: usize = min_dim;
    while ii > 0 {
        ii = ii - 1;
        let i = ii;

        let axis_start = i * nrows + i;
        let axis_len = nrows - i;

        let mut axis = vec![0.0; axis_len];
        let mut j: usize = 0;
        while j < axis_len {
            axis[j] = compact[axis_start + j];
            j += 1;
        }

        // Normalize the sign convention of the stored axis (unit vector).
        if axis[0] < 0.0 {
            let mut j: usize = 0;
            while j < axis_len {
                axis[j] = -axis[j];
                j += 1;
            }
        }

        let sign = if diag[i] < 0.0 { -1.0 } else { 1.0 };

        let mut col: usize = i;
        while col < min_dim {
            let col_start = col * nrows + i;

            let mut dot: f64 = 0.0;
            let mut j: usize = 0;
            while j < axis_len {
                dot = dot + axis[j] * q[col_start + j];
                j += 1;
            }

            let factor = dot * -2.0;
            let mut j: usize = 0;
            while j < axis_len {
                q[col_start + j] = sign * (q[col_start + j] + factor * axis[j]);
                j += 1;
            }

            col += 1;
        }
    }

    q
}

/// Reconstruct the R factor (min_dim × ncols, column-major, upper triangular).
fn build_r(compact: &[f64], diag: &[f64], nrows: usize, ncols: usize) -> Vec<f64> {
    let min_dim = if nrows < ncols { nrows } else { ncols };

    let mut r = vec![0.0; min_dim * ncols];

    let mut col: usize = 0;
    while col < ncols {
        let mut row: usize = 0;
        while row < min_dim {
            if row <= col {
                r[row + col * min_dim] = compact[row + col * nrows];
            }
            row += 1;
        }
        col += 1;
    }

    // Diagonal entries are the absolute values of the signed norms.
    let mut i: usize = 0;
    while i < min_dim {
        r[i + i * min_dim] = if diag[i] < 0.0 { -diag[i] } else { diag[i] };
        i += 1;
    }

    r
}

/// Compute the reduced QR decomposition of an m×n matrix (column-major).
///
/// Returns Q (m×k, orthonormal columns) and R (k×n, upper triangular) with
/// k = min(m, n), such that A = Q · R.
///
/// * `matrix` — flattened m×n matrix in column-major order
/// * `nrows` — m
/// * `ncols` — n
pub fn qr_decomposition(matrix: &[f64], nrows: usize, ncols: usize) -> Qr {
    let min_nm = if nrows < ncols { nrows } else { ncols };

    if min_nm == 0 {
        return Qr {
            q: Vec::new(),
            r: Vec::new(),
        };
    }

    // Compute the compact form (Householder vectors + signed norms).
    let mut compact = matrix.to_vec();
    let mut diag: Vec<f64> = Vec::new();

    let mut icol: usize = 0;
    while icol < min_nm {
        let (norm, compact2) = clear_column(compact, nrows, ncols, icol);
        compact = compact2;
        diag.push(norm);
        icol += 1;
    }

    // Reconstruct the explicit Q and R factors.
    let q = build_q(&compact, &diag, nrows, ncols);
    let r = build_r(&compact, &diag, nrows, ncols);

    Qr { q, r }
}
