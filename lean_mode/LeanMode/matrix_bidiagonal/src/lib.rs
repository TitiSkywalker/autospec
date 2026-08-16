//! Bidiagonalization using Householder reflections.
//!
//! Extracted from nalgebra's `Bidiagonal::new` + `u` + `d` + `v_t` (via
//! `householder::clear_column_unchecked` / `clear_row_unchecked`) and refined
//! for Aeneas compatibility (while loops, index-based access, no
//! iterators/closures).
//!
//! Decomposes an m×n matrix A (with m ≥ n) into A = U · B · Vᵀ, where U (m×n)
//! and V (n×n) are orthogonal (Uᵀ·U = I, Vᵀ·V = I) and B (n×n) is upper
//! bidiagonal — every entry off the diagonal and first superdiagonal is zero.
//!
//! This is the *upper* bidiagonal case of nalgebra's implementation (the
//! `upper_diagonal` branch, entered when `nrows >= ncols`); the lower case
//! (`nrows < ncols`) is excluded via `assert!`.
//!
//! All matrices are stored in **column-major** order: element (row, col) is
//! at index `row + col * nrows` (U and the working matrix use `nrows`; B uses
//! its own dimension; Vᵀ uses `n`).

/// Result of the upper bidiagonalization.
pub struct Bidiagonal {
    /// Orthogonal factor U (m×n, column-major; orthonormal columns).
    pub u: Vec<f64>,
    /// Upper bidiagonal matrix B (n×n, column-major).
    pub b: Vec<f64>,
    /// Orthogonal factor Vᵀ (n×n, column-major; orthonormal rows).
    pub v_t: Vec<f64>,
}

/// Compute the Householder reflection axis for a vector, in place.
///
/// Given `axis[0..len]`, computes the reflection that zeros out all entries
/// except the first.  The vector is normalized to unit length.
///
/// Returns `(signed_norm, not_zero)` where `signed_norm` is the signed
/// Euclidean norm of the original vector.
fn reflection_axis(axis: &mut Vec<f64>, len: usize) -> (f64, bool) {
    if len == 0 {
        return (0.0, false);
    }

    // Squared Euclidean norm of the vector.
    let mut sq_norm: f64 = 0.0;
    let mut i: usize = 0;
    while i < len {
        sq_norm = sq_norm + axis[i] * axis[i];
        i += 1;
    }
    let norm = sq_norm.sqrt();
    if norm == 0.0 {
        return (0.0, false);
    }

    let first = axis[0];
    let sign = if first < 0.0 { -1.0 } else { 1.0 };
    let modulus = if first < 0.0 { -first } else { first };
    let signed_norm = sign * norm;

    // ‖v‖² = 2·(‖x‖² + |x₀|·‖x‖) for v = x + sign(x₀)·‖x‖·e₁.
    let factor = (sq_norm + modulus * norm) * 2.0;
    if factor == 0.0 {
        return (signed_norm, false);
    }

    axis[0] = first + signed_norm;

    let factor_sqrt = factor.sqrt();
    let mut i: usize = 0;
    while i < len {
        axis[i] = axis[i] / factor_sqrt;
        i += 1;
    }

    (-signed_norm, true)
}

/// Zero out the sub-diagonal of column `icol` with a Householder reflection
/// and apply it (one-sided, left multiply) to the trailing columns.
///
/// Returns the signed norm of the column and the updated compact matrix.
fn clear_column(
    mut matrix: Vec<f64>,
    nrows: usize,
    ncols: usize,
    icol: usize,
) -> (f64, Vec<f64>) {
    let axis_start = icol * nrows + icol;
    let axis_len = nrows - icol;

    let mut axis = vec![0.0; axis_len];
    let mut i: usize = 0;
    while i < axis_len {
        axis[i] = matrix[axis_start + i];
        i += 1;
    }

    let (reflection_norm, not_zero) = reflection_axis(&mut axis, axis_len);

    // Store the (unit) Householder vector back into column `icol`.
    let mut i: usize = 0;
    while i < axis_len {
        matrix[axis_start + i] = axis[i];
        i += 1;
    }

    if not_zero {
        let sign = if reflection_norm < 0.0 { -1.0 } else { 1.0 };
        let factor_scale = -2.0 * sign;

        // Apply H = I - 2·v·vᵀ to columns icol+1..ncols, rows icol..nrows.
        let mut col: usize = icol + 1;
        while col < ncols {
            let col_start = col * nrows + icol;
            let mut dot: f64 = 0.0;
            let mut i: usize = 0;
            while i < axis_len {
                dot = dot + axis[i] * matrix[col_start + i];
                i += 1;
            }
            let factor = factor_scale * dot;
            let mut i: usize = 0;
            while i < axis_len {
                let idx = col_start + i;
                let new_val = sign * matrix[idx] + factor * axis[i];
                matrix[idx] = new_val;
                i += 1;
            }
            col += 1;
        }
    }

    (reflection_norm, matrix)
}

/// Zero out the super-diagonal of row `irow` with a Householder reflection
/// and apply it (one-sided, right multiply) to the trailing rows.
///
/// Returns the signed norm of the row and the updated compact matrix.
fn clear_row(
    mut matrix: Vec<f64>,
    nrows: usize,
    ncols: usize,
    irow: usize,
) -> (f64, Vec<f64>) {
    let axis_len = ncols - irow - 1;

    // Extract row `irow`, columns irow+1..ncols.
    let mut axis = vec![0.0; axis_len];
    let mut t: usize = 0;
    while t < axis_len {
        axis[t] = matrix[irow + (irow + 1 + t) * nrows];
        t += 1;
    }

    let (reflection_norm, not_zero) = reflection_axis(&mut axis, axis_len);

    if not_zero {
        let sign = if reflection_norm < 0.0 { -1.0 } else { 1.0 };
        let factor_scale = -2.0 * sign;

        // Apply H = I - 2·v·vᵀ to rows irow+1..nrows, columns irow+1..ncols.
        let mut row: usize = irow + 1;
        while row < nrows {
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < axis_len {
                dot = dot + matrix[row + (irow + 1 + t) * nrows] * axis[t];
                t += 1;
            }
            let factor = factor_scale * dot;
            let mut t: usize = 0;
            while t < axis_len {
                let idx = row + (irow + 1 + t) * nrows;
                let new_val = sign * matrix[idx] + factor * axis[t];
                matrix[idx] = new_val;
                t += 1;
            }
            row += 1;
        }
    }

    // Store the (unit) Householder vector back into row `irow`,
    // columns irow+1..ncols.
    let mut t: usize = 0;
    while t < axis_len {
        matrix[irow + (irow + 1 + t) * nrows] = axis[t];
        t += 1;
    }

    (reflection_norm, matrix)
}

/// Reconstruct the orthogonal factor U (m×n, column-major) by applying the
/// column Householder reflections to the identity in reverse order.
fn assemble_u(uv: &[f64], diagonal: &[f64], nrows: usize, d: usize) -> Vec<f64> {
    let mut u = vec![0.0; nrows * d];
    let mut i: usize = 0;
    while i < d {
        u[i + i * nrows] = 1.0;
        i += 1;
    }

    // Apply reflections in reverse order: i = d-1 down to 0.
    let mut ii: usize = d;
    while ii > 0 {
        ii -= 1;
        let i = ii;
        let axis_len = nrows - i;

        // Skip a zero-magnitude axis (the original column was already zero).
        let mut sq: f64 = 0.0;
        let mut t: usize = 0;
        while t < axis_len {
            let v = uv[(i + t) + i * nrows];
            sq = sq + v * v;
            t += 1;
        }
        let is_nonzero = sq != 0.0;
        if is_nonzero {
            let sign = if diagonal[i] < 0.0 { -1.0 } else { 1.0 };
            let factor_scale = -2.0 * sign;

            // Apply H = I - 2·v·vᵀ to columns i..d-1, rows i..nrows.
            let mut col: usize = i;
            while col < d {
                let mut dot: f64 = 0.0;
                let mut t: usize = 0;
                while t < axis_len {
                    dot = dot + uv[(i + t) + i * nrows] * u[(i + t) + col * nrows];
                    t += 1;
                }
                let factor = factor_scale * dot;
                let mut t: usize = 0;
                while t < axis_len {
                    let idx = (i + t) + col * nrows;
                    let new_val = sign * u[idx] + factor * uv[(i + t) + i * nrows];
                    u[idx] = new_val;
                    t += 1;
                }
                col += 1;
            }
        }
    }

    u
}

/// Reconstruct the orthogonal factor Vᵀ (n×n, column-major) by applying the
/// row Householder reflections to the identity in reverse order.
fn assemble_v_t(
    uv: &[f64],
    off_diagonal: &[f64],
    nrows: usize,
    ncols: usize,
    d: usize,
) -> Vec<f64> {
    let mut v_t = vec![0.0; d * ncols];
    let mut i: usize = 0;
    while i < d {
        v_t[i + i * d] = 1.0;
        i += 1;
    }

    // Apply reflections in reverse order: i = d-2 down to 0.
    let mut ii: usize = d - 1;
    while ii > 0 {
        ii -= 1;
        let i = ii;
        let axis_len = ncols - i - 1;

        // Skip a zero-magnitude axis (the original row was already zero).
        let mut sq: f64 = 0.0;
        let mut t: usize = 0;
        while t < axis_len {
            let v = uv[i + (i + 1 + t) * nrows];
            sq = sq + v * v;
            t += 1;
        }
        let is_nonzero = sq != 0.0;
        if is_nonzero {
            let sign = if off_diagonal[i] < 0.0 { -1.0 } else { 1.0 };
            let factor_scale = -2.0 * sign;

            // Apply H = I - 2·v·vᵀ to rows i..d-1, columns i+1..ncols.
            let mut row: usize = i;
            while row < d {
                let mut dot: f64 = 0.0;
                let mut t: usize = 0;
                while t < axis_len {
                    dot = dot
                        + uv[i + (i + 1 + t) * nrows] * v_t[row + (i + 1 + t) * d];
                    t += 1;
                }
                let factor = factor_scale * dot;
                let mut t: usize = 0;
                while t < axis_len {
                    let idx = row + (i + 1 + t) * d;
                    let new_val = sign * v_t[idx] + factor * uv[i + (i + 1 + t) * nrows];
                    v_t[idx] = new_val;
                    t += 1;
                }
                row += 1;
            }
        }
    }

    v_t
}

/// Reconstruct the upper bidiagonal matrix B (n×n, column-major) from the
/// diagonal and off-diagonal signed norms.
fn unpack_b(diagonal: &[f64], off_diagonal: &[f64], d: usize) -> Vec<f64> {
    let mut b = vec![0.0; d * d];

    // Diagonal entries B[i, i] = |diagonal[i]|.
    let mut i: usize = 0;
    while i < d {
        let diag = diagonal[i];
        let abs_diag = if diag < 0.0 { -diag } else { diag };
        b[i + i * d] = abs_diag;
        i += 1;
    }

    // Super-diagonal entries B[i, i+1] = |off_diagonal[i]|.
    let mut i: usize = 0;
    while i < d - 1 {
        let od = off_diagonal[i];
        let abs_od = if od < 0.0 { -od } else { od };
        b[i + (i + 1) * d] = abs_od;
        i += 1;
    }

    b
}

/// Compute the upper bidiagonalization of an m×n matrix (column-major) with
/// m ≥ n.
///
/// Returns U (m×n, orthonormal columns), B (n×n, upper bidiagonal) and Vᵀ
/// (n×n, orthonormal rows) such that A = U · B · Vᵀ.
///
/// * `matrix` — flattened m×n matrix in column-major order
/// * `nrows` — m
/// * `ncols` — n
///
/// # Panics
///
/// Panics if `nrows < ncols` (the lower-bidiagonal case is not implemented).
pub fn bidiagonal_decomposition(matrix: &[f64], nrows: usize, ncols: usize) -> Bidiagonal {
    assert!(
        nrows >= ncols,
        "bidiagonalization: only the upper (nrows >= ncols) case is supported"
    );

    let d = ncols;
    if d == 0 {
        return Bidiagonal {
            u: Vec::new(),
            b: Vec::new(),
            v_t: Vec::new(),
        };
    }

    // Compute the compact form (Householder vectors + signed norms).
    let mut uv = matrix.to_vec();
    let mut diagonal: Vec<f64> = vec![0.0; d];
    let mut off_diagonal: Vec<f64> = vec![0.0; d - 1];

    let mut ite: usize = 0;
    while ite < d - 1 {
        let (dnorm, uv2) = clear_column(uv, nrows, ncols, ite);
        uv = uv2;
        diagonal[ite] = dnorm;

        let (onorm, uv3) = clear_row(uv, nrows, ncols, ite);
        uv = uv3;
        off_diagonal[ite] = onorm;

        ite += 1;
    }

    let (dnorm, uv2) = clear_column(uv, nrows, ncols, d - 1);
    uv = uv2;
    diagonal[d - 1] = dnorm;

    // Reconstruct the explicit U, B and Vᵀ factors.
    let u = assemble_u(&uv, &diagonal, nrows, d);
    let b = unpack_b(&diagonal, &off_diagonal, d);
    let v_t = assemble_v_t(&uv, &off_diagonal, nrows, ncols, d);

    Bidiagonal { u, b, v_t }
}
