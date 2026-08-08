//! Bidiagonal decomposition using Householder reflections.
//!
//! Extracted from nalgebra's `Bidiagonal::new` method and refined for Aeneas
//! compatibility (while loops, index-based access, no slicing).
//!
//! Decomposes a general m×n matrix M into M = U · B · Vᵀ where B is
//! bidiagonal and U, V are orthogonal (products of Householder reflections).
//!
//! The matrix is stored in **column-major** order: element (row, col) is at
//! index ``row + col * nrows``.

// ── basic vector ops ──────────────────────────────────────────────────────────

/// ``v[0..len] · v[0..len]`` — squared Euclidean norm.
pub fn norm_squared(v: &[f64], len: usize) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: usize = 0;
    while i < len {
        sum = sum + v[i] * v[i];
        i = i + 1;
    }
    sum
}

/// Dot product of two equal-length vectors.
pub fn dot(v1: &[f64], v2: &[f64], len: usize) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: usize = 0;
    while i < len {
        sum = sum + v1[i] * v2[i];
        i = i + 1;
    }
    sum
}

/// ``y[i] += alpha * x[i]``  for i in 0..len.
pub fn axpy(alpha: f64, x: &[f64], y: &mut [f64], len: usize) {
    let mut i: usize = 0;
    while i < len {
        y[i] = y[i] + alpha * x[i];
        i = i + 1;
    }
}

/// ``v[i] *= alpha``  for i in 0..len.
pub fn scale(alpha: f64, v: &mut [f64], len: usize) {
    let mut i: usize = 0;
    while i < len {
        v[i] = v[i] * alpha;
        i = i + 1;
    }
}

/// Check whether `|x|` is below a small threshold.
pub fn is_zero(x: f64) -> bool {
    let abs_x = if x < 0.0 { -x } else { x };
    abs_x < 1e-15
}

/// Signum: 1.0, -1.0, or 0.0.
pub fn signum(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

// ── column / row copy helpers ─────────────────────────────────────────────────
// All indices are relative to the full matrix in column-major storage.

/// Copy `matrix[row_start..row_end][col]` → `target[0..row_end-row_start]`.
pub fn copy_column(
    matrix: &[f64],
    nrows: usize,
    col: usize,
    row_start: usize,
    row_end: usize,
    target: &mut [f64],
) {
    let mut i: usize = 0;
    while i < row_end - row_start {
        target[i] = matrix[(row_start + i) + col * nrows];
        i = i + 1;
    }
}

/// Copy `source[0..len]` → `matrix[row_start..][col]`.
pub fn write_column(
    source: &[f64],
    matrix: &mut [f64],
    nrows: usize,
    col: usize,
    row_start: usize,
    len: usize,
) {
    let mut i: usize = 0;
    while i < len {
        matrix[(row_start + i) + col * nrows] = source[i];
        i = i + 1;
    }
}

/// Copy `matrix[row][col_start..col_end]` → `target[0..col_end-col_start]`.
pub fn copy_row(
    matrix: &[f64],
    nrows: usize,
    row: usize,
    col_start: usize,
    col_end: usize,
    target: &mut [f64],
) {
    let mut j: usize = 0;
    while j < col_end - col_start {
        target[j] = matrix[row + (col_start + j) * nrows];
        j = j + 1;
    }
}

/// Copy `source[0..len]` → `matrix[row][col_start..]`.
pub fn write_row(
    source: &[f64],
    matrix: &mut [f64],
    nrows: usize,
    row: usize,
    col_start: usize,
    len: usize,
) {
    let mut j: usize = 0;
    while j < len {
        matrix[row + (col_start + j) * nrows] = source[j];
        j = j + 1;
    }
}

// ── Householder helpers ───────────────────────────────────────────────────────

/// Compute the Householder reflection axis (sqrt-free version).
///
/// Given a vector `axis[0..len]`, computes the reflection that zeros out
/// all entries except the first.  The axis is **not** normalized because
/// the Householder formula ``H = I - 2·v·vᵀ / (vᵀ·v)`` is scale-invariant.
///
/// Returns ``(signed_norm, not_zero)`` where `signed_norm` is the signed
/// Euclidean norm of the original vector (used as the diagonal /
/// off-diagonal entry).
///
/// This is based on nalgebra's ``reflection_axis_mut`` with the
/// normalization step removed — it does not require ``sqrt``, making it
/// fully computable on ``Rat`` after translation.
pub fn reflection_axis(axis: &mut [f64], len: usize) -> (f64, bool) {
    let sq_norm = norm_squared(axis, len);

    if is_zero(sq_norm) {
        return (0.0, false);
    }

    let norm = sq_norm.sqrt();
    let signed_norm = signum(axis[0]) * norm;

    // Build the (un-normalized) Householder vector:
    //   v₁ = x₁ + sign(x₁)·‖x‖,   vᵢ = xᵢ (i > 1)
    // The reflection  H = I - β·v·vᵀ  with  β = 2/(vᵀv)  is independent
    // of the scaling of v, so we skip the usual normalization step.
    axis[0] = axis[0] + signed_norm;
    (-signed_norm, true)
}

/// Apply a Householder reflection to columns of a submatrix.
///
/// ``sub`` is in column-major order with stride `nrows` (rows 0..nrows).
/// The first column is column `col_start` of the original matrix.
pub fn reflect_cols(
    sub: &mut [f64],
    nrows: usize,
    ncols: usize,
    row_start: usize,
    axis: &[f64],
    sign: f64,
) {
    let m_two = sign * (-2.0);
    let axis_len = nrows - row_start;
    let mut j: usize = 0;
    while j < ncols {
        let col_base = j * nrows;
        let mut dot_col: f64 = 0.0;
        let mut i: usize = 0;
        while i < axis_len {
            dot_col = dot_col + axis[i] * sub[col_base + row_start + i];
            i = i + 1;
        }
        let factor = dot_col * m_two;
        let mut i: usize = 0;
        while i < axis_len {
            sub[col_base + row_start + i] =
                sub[col_base + row_start + i] + factor * axis[i];
            i = i + 1;
        }
        j = j + 1;
    }
}

/// Apply a Householder reflection to rows of a submatrix.
///
/// ``sub`` is in column-major order with stride `nrows`.
/// The first column is column `col_start` of the original matrix.
/// ``work[0..nrows]`` is scratch space.
pub fn reflect_rows(
    sub: &mut [f64],
    nrows: usize,
    ncols: usize,
    row_start: usize,
    axis: &[f64],
    work: &mut [f64],
    sign: f64,
) {
    let m_two = sign * (-2.0);
    let axis_len = ncols;
    let bottom_nrows = nrows - row_start;

    let mut i: usize = 0;
    while i < bottom_nrows {
        work[i] = 0.0;
        i = i + 1;
    }

    let mut j: usize = 0;
    while j < axis_len {
        let mut i: usize = 0;
        while i < bottom_nrows {
            work[i] = work[i] + sub[(row_start + i) + j * nrows] * axis[j];
            i = i + 1;
        }
        j = j + 1;
    }

    let mut j: usize = 0;
    while j < axis_len {
        let mut i: usize = 0;
        while i < bottom_nrows {
            sub[(row_start + i) + j * nrows] =
                sub[(row_start + i) + j * nrows] + m_two * work[i] * axis[j];
            i = i + 1;
        }
        j = j + 1;
    }
}

// ── column / row clearing ─────────────────────────────────────────────────────

/// Eliminate sub-diagonal entries in column `icol` using a Householder
/// reflection and apply it to the trailing columns.
///
/// Returns the diagonal entry (the signed norm of the column vector).
pub fn clear_column(
    matrix: &mut [f64],
    nrows: usize,
    ncols: usize,
    icol: usize,
    shift: usize,
    axis_work: &mut [f64],
    sub_work: &mut [f64],
) -> f64 {
    let axis_len = nrows - (icol + shift);
    copy_column(matrix, nrows, icol, icol + shift, nrows, axis_work);

    let (reflection_norm, not_zero) = reflection_axis(axis_work, axis_len);
    write_column(axis_work, matrix, nrows, icol, icol + shift, axis_len);

    if not_zero {
        let sign = signum(reflection_norm);
        let right_ncols = ncols - (icol + 1);
        if right_ncols > 0 {
            let bottom_start = icol + shift;
            let bottom_nrows = nrows - bottom_start;
            let mut j: usize = 0;
            while j < right_ncols {
                let mut i: usize = 0;
                while i < bottom_nrows {
                    sub_work[i + j * bottom_nrows] =
                        matrix[(bottom_start + i) + (icol + 1 + j) * nrows];
                    i = i + 1;
                }
                j = j + 1;
            }

            reflect_cols(sub_work, bottom_nrows, right_ncols, 0, axis_work, sign);

            let mut j: usize = 0;
            while j < right_ncols {
                let mut i: usize = 0;
                while i < bottom_nrows {
                    matrix[(bottom_start + i) + (icol + 1 + j) * nrows] =
                        sub_work[i + j * bottom_nrows];
                    i = i + 1;
                }
                j = j + 1;
            }
        }
    }

    reflection_norm
}

/// Eliminate super-diagonal entries in row `irow` using a Householder
/// reflection and apply it to the trailing rows.
///
/// Returns the off-diagonal entry (the signed norm of the row vector).
pub fn clear_row(
    matrix: &mut [f64],
    nrows: usize,
    ncols: usize,
    irow: usize,
    shift: usize,
    axis_work: &mut [f64],
    work: &mut [f64],
    sub_work: &mut [f64],
) -> f64 {
    let axis_len = ncols - (irow + shift);
    copy_row(matrix, nrows, irow, irow + shift, ncols, axis_work);

    let (reflection_norm, not_zero) = reflection_axis(axis_work, axis_len);
    write_row(axis_work, matrix, nrows, irow, irow + shift, axis_len);

    if not_zero {
        let sign = signum(reflection_norm);

        let bottom_start = irow + 1;
        let bottom_nrows = nrows - bottom_start;
        let right_start = irow + shift;
        let right_ncols = ncols - right_start;

        if bottom_nrows > 0 && right_ncols > 0 {
            let mut j: usize = 0;
            while j < right_ncols {
                let mut i: usize = 0;
                while i < bottom_nrows {
                    sub_work[i + j * bottom_nrows] =
                        matrix[(bottom_start + i) + (right_start + j) * nrows];
                    i = i + 1;
                }
                j = j + 1;
            }

            reflect_rows(sub_work, bottom_nrows, right_ncols, 0, axis_work, work, sign);

            let mut j: usize = 0;
            while j < right_ncols {
                let mut i: usize = 0;
                while i < bottom_nrows {
                    matrix[(bottom_start + i) + (right_start + j) * nrows] =
                        sub_work[i + j * bottom_nrows];
                    i = i + 1;
                }
                j = j + 1;
            }
        }
    }

    reflection_norm
}

// ── main decomposition ────────────────────────────────────────────────────────

/// Bidiagonalize a general m×n matrix using Householder reflections.
///
/// * `matrix` — column-major m×n matrix (modified in place to store
///   the Householder vectors)
/// * `nrows` — m
/// * `ncols` — n
/// * `diagonal` — pre-allocated, length min(m, n)
/// * `off_diagonal` — pre-allocated, length min(m, n) - 1
/// * `col_axis_work` — scratch, length m
/// * `row_axis_work` — scratch, length n
/// * `work` — scratch, length m
/// * `sub_work` — scratch, length m * n
///
/// Returns whether the bidiagonal is upper-diagonal (m ≥ n).
pub fn bidiagonal_decompose(
    matrix: &mut [f64],
    nrows: usize,
    ncols: usize,
    diagonal: &mut [f64],
    off_diagonal: &mut [f64],
    col_axis_work: &mut [f64],
    row_axis_work: &mut [f64],
    work: &mut [f64],
    sub_work: &mut [f64],
) -> bool {
    assert!(nrows > 0 && ncols > 0);

    let min_nm = if nrows < ncols { nrows } else { ncols };
    let upper_diagonal = nrows >= ncols;

    if upper_diagonal {
        let mut ite: usize = 0;
        while ite < min_nm - 1 {
            diagonal[ite] = clear_column(
                matrix, nrows, ncols, ite, 0, col_axis_work, sub_work,
            );
            off_diagonal[ite] = clear_row(
                matrix, nrows, ncols, ite, 1, row_axis_work, work, sub_work,
            );
            ite = ite + 1;
        }
        diagonal[min_nm - 1] = clear_column(
            matrix, nrows, ncols, min_nm - 1, 0, col_axis_work, sub_work,
        );
    } else {
        let mut ite: usize = 0;
        while ite < min_nm - 1 {
            diagonal[ite] = clear_row(
                matrix, nrows, ncols, ite, 0, row_axis_work, work, sub_work,
            );
            off_diagonal[ite] = clear_column(
                matrix, nrows, ncols, ite, 1, col_axis_work, sub_work,
            );
            ite = ite + 1;
        }
        diagonal[min_nm - 1] = clear_row(
            matrix, nrows, ncols, min_nm - 1, 0, row_axis_work, work, sub_work,
        );
    }

    upper_diagonal
}

/// Convenience wrapper: allocates workspace and returns owned vectors.
///
/// Returns ``(diagonal, off_diagonal, upper_diagonal, uv)`` where
/// * `diagonal` — diagonal entries of B
/// * `off_diagonal` — off-diagonal entries of B
/// * `upper_diagonal` — true if m ≥ n
/// * `uv` — matrix with Householder vectors (upper triangle = U, lower = V)
pub fn bidiagonal(
    matrix: &mut [f64],
    nrows: usize,
    ncols: usize,
) -> (Vec<f64>, Vec<f64>, bool, Vec<f64>) {
    let min_nm = if nrows < ncols { nrows } else { ncols };
    let mut diagonal = vec![0.0; min_nm];
    let mut off_diagonal = vec![0.0; if min_nm > 0 { min_nm - 1 } else { 0 }];
    let mut col_axis_work = vec![0.0; nrows];
    let mut row_axis_work = vec![0.0; ncols];
    let mut work = vec![0.0; nrows];
    let mut sub_work = vec![0.0; nrows * ncols];

    let uv = matrix.to_vec();

    let upper_diagonal = bidiagonal_decompose(
        matrix, nrows, ncols, &mut diagonal, &mut off_diagonal,
        &mut col_axis_work, &mut row_axis_work, &mut work, &mut sub_work,
    );

    (diagonal, off_diagonal, upper_diagonal, uv)
}
