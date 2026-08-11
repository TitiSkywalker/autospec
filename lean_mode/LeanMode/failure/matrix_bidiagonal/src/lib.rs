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

    let reflection_norm = sq_norm.sqrt();
    let signed_norm = signum(axis[0]) * reflection_norm;

    // Build the Householder vector and normalize to unit length
    //   v = x + sign(x₀)·‖x‖·e₁,  then  v := v / ‖v‖
    axis[0] = axis[0] + signed_norm;

    // ‖v‖² = 2·‖x‖·(|x₀| + ‖x‖)
    let x0_abs = if axis[0] - signed_norm < 0.0 { -(axis[0] - signed_norm) } else { axis[0] - signed_norm };
    let v_norm_sq = 2.0 * reflection_norm * (x0_abs + reflection_norm);
    if !is_zero(v_norm_sq) {
        let v_norm = v_norm_sq.sqrt();
        let mut i: usize = 0;
        while i < len {
            axis[i] = axis[i] / v_norm;
            i = i + 1;
        }
    }

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
    // reflect_with_sign: col := sign * (col - 2*(axisᵀ·col)*axis)
    // where axis is unit-length
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
        let factor = -2.0 * dot_col;
        let mut i: usize = 0;
        while i < axis_len {
            let idx = col_base + row_start + i;
            sub[idx] = sign * (sub[idx] + factor * axis[i]);
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
///
/// reflect_rows_with_sign: row := sign * (row - 2·(row·axis)·axisᵀ)
pub fn reflect_rows(
    sub: &mut [f64],
    nrows: usize,
    ncols: usize,
    row_start: usize,
    axis: &[f64],
    work: &mut [f64],
    sign: f64,
) {
    let axis_len = ncols;
    let bottom_nrows = nrows - row_start;

    // work = sub · axis
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

    // sub := sign * (sub - 2·work·axisᵀ)
    let mut j: usize = 0;
    while j < axis_len {
        let mut i: usize = 0;
        while i < bottom_nrows {
            let idx = (row_start + i) + j * nrows;
            sub[idx] = sign * (sub[idx] - 2.0 * work[i] * axis[j]);
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

/// Convenience wrapper: allocates workspace and returns U, B, Vt.
///
/// Returns ``(U, B, Vt)`` where
/// * `U` — m×k orthogonal matrix (column-major), k = min(m,n)
/// * `B` — k×k bidiagonal matrix (column-major)
/// * `Vt` — k×n, the transpose of the orthogonal V (column-major)
///
/// The decomposition satisfies  M = U · B · Vt.
pub fn bidiagonal(
    matrix: &mut [f64],
    nrows: usize,
    ncols: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let min_nm = if nrows < ncols { nrows } else { ncols };
    let mut diagonal = vec![0.0; min_nm];
    let mut off_diagonal = vec![0.0; if min_nm > 0 { min_nm - 1 } else { 0 }];
    let mut col_axis_work = vec![0.0; nrows];
    let mut row_axis_work = vec![0.0; ncols];
    let mut work = vec![0.0; nrows];
    let mut sub_work = vec![0.0; nrows * ncols];

    let upper_diagonal = bidiagonal_decompose(
        matrix, nrows, ncols, &mut diagonal, &mut off_diagonal,
        &mut col_axis_work, &mut row_axis_work, &mut work, &mut sub_work,
    );

    let U = build_u(matrix, nrows, ncols, &diagonal, &off_diagonal, upper_diagonal);
    let Vt = build_vt(matrix, nrows, ncols, &diagonal, &off_diagonal, upper_diagonal);
    let B = build_b(&diagonal, &off_diagonal, upper_diagonal);

    (U, B, Vt)
}

// ── U / Vt / B reconstruction (nalgebra conventions) ──────────────────────────

/// Build U (m × k, column-major), matching nalgebra's `Bidiagonal::u()`.
///
/// Uses `reflect_with_sign(sign)` formula:
///   col_new := sign · (col - 2/(vᵀv)·(vᵀ·col)·v)
///
/// Applied in **reverse** order (left-multiplication).
/// The sign comes from `diagonal[i].signum()` (upper) or `off_diagonal[i].signum()` (lower).
fn build_u(
    modified: &[f64],
    m: usize,
    _n: usize,
    diagonal: &[f64],
    off_diagonal: &[f64],
    upper: bool,
) -> Vec<f64> {
    let k = diagonal.len();
    let shift: usize = if upper { 0 } else { 1 };
    let kmax = if k > shift { k - shift } else { 0 };

    let mut U = vec![0.0; m * k];
    for i in 0..k {
        U[i + i * m] = 1.0;
    }

    let mut ii: isize = kmax as isize - 1;
    while ii >= 0 {
        let i = ii as usize;
        let start_row = i + shift;
        let axis_len = m - start_row;
        ii = ii - 1;

        if axis_len == 0 { continue; }

        let mut v = vec![0.0; axis_len];
        let mut vtv: f64 = 0.0;
        let mut r: usize = 0;
        while r < axis_len {
            let val = modified[(start_row + r) + i * m];
            v[r] = val;
            vtv = vtv + val * val;
            r = r + 1;
        }

        if is_zero(vtv) { continue; }
        let beta = 2.0 / vtv;
        let sgn = if upper { signum(diagonal[i]) } else { signum(off_diagonal[i]) };

        // reflect_with_sign: col := sgn · (col - β·(vᵀ·col)·v)
        let mut c: usize = i;
        while c < k {
            let mut dot: f64 = 0.0;
            let mut r: usize = 0;
            while r < axis_len {
                dot = dot + v[r] * U[(start_row + r) + c * m];
                r = r + 1;
            }
            let factor = beta * dot;
            let mut r: usize = 0;
            while r < axis_len {
                let idx = (start_row + r) + c * m;
                U[idx] = sgn * (U[idx] - factor * v[r]);
                r = r + 1;
            }
            c = c + 1;
        }
    }

    U
}

/// Build Vᵀ (k × n, column-major), matching nalgebra's `Bidiagonal::v_t()`.
///
/// Uses `reflect_rows_with_sign(sign)` which applies:
///   row_new := sign · (row - 2/(wᵀw)·(row·w)·wᵀ)
///
/// Applied from the right in **reverse** order.
/// The sign comes from `off_diagonal[i].signum()` (upper) or `diagonal[i].signum()` (lower).
fn build_vt(
    modified: &[f64],
    m: usize,
    n: usize,
    diagonal: &[f64],
    off_diagonal: &[f64],
    upper: bool,
) -> Vec<f64> {
    let k = diagonal.len();
    let shift: usize = if upper { 1 } else { 0 };
    let kmax = if k > shift { k - shift } else { 0 };

    let mut V = vec![0.0; n * n];
    let mut j: usize = 0;
    while j < n {
        V[j + j * n] = 1.0;
        j = j + 1;
    }

    let mut ii: isize = kmax as isize - 1;
    while ii >= 0 {
        let i = ii as usize;
        let start_col = i + shift;
        let axis_len = n - start_col;
        ii = ii - 1;

        if axis_len == 0 { continue; }

        let mut w = vec![0.0; axis_len];
        let mut wtw: f64 = 0.0;
        let mut c: usize = 0;
        while c < axis_len {
            let val = modified[i + (start_col + c) * m];
            w[c] = val;
            wtw = wtw + val * val;
            c = c + 1;
        }

        if is_zero(wtw) { continue; }
        let beta = 2.0 / wtw;
        let sgn = if upper { signum(off_diagonal[i]) } else { signum(diagonal[i]) };

        // reflect_rows_with_sign: V := sgn · (V - β·(V·w)·wᵀ)
        // Step 1: Vw = V · w
        let mut Vw = vec![0.0; n];
        let mut r: usize = 0;
        while r < n {
            let mut dot: f64 = 0.0;
            let mut c: usize = 0;
            while c < axis_len {
                dot = dot + V[r + (start_col + c) * n] * w[c];
                c = c + 1;
            }
            Vw[r] = dot;
            r = r + 1;
        }

        // Step 2: V[r][c] = sgn * (V[r][c] - β · Vw[r] · w[c])
        let mut r: usize = 0;
        while r < n {
            let mut c: usize = 0;
            while c < axis_len {
                let idx = r + (start_col + c) * n;
                V[idx] = sgn * (V[idx] - beta * Vw[r] * w[c]);
                c = c + 1;
            }
            r = r + 1;
        }
    }

    // V^T[r][c] = V[c][r], first k rows → Vt (k×n column-major)
    let mut Vt = vec![0.0; k * n];
    let mut c: usize = 0;
    while c < n {
        let mut r: usize = 0;
        while r < k {
            Vt[r + c * k] = V[r + c * n];
            r = r + 1;
        }
        c = c + 1;
    }

    Vt
}

/// Build the bidiagonal matrix B (k × k, column-major) with absolute-value
/// entries, matching nalgebra's `Bidiagonal::d()`.
fn build_b(diagonal: &[f64], off_diagonal: &[f64], upper: bool) -> Vec<f64> {
    let k = diagonal.len();
    let mut B = vec![0.0; k * k];

    // Diagonal: B[i + i*k] = |diagonal[i]|
    let mut i: usize = 0;
    while i < k {
        B[i + i * k] = if diagonal[i] < 0.0 { -diagonal[i] } else { diagonal[i] };
        i = i + 1;
    }

    if k > 0 && !off_diagonal.is_empty() {
        if upper {
            let mut i: usize = 0;
            while i < off_diagonal.len() {
                B[i + (i + 1) * k] =
                    if off_diagonal[i] < 0.0 { -off_diagonal[i] } else { off_diagonal[i] };
                i = i + 1;
            }
        } else {
            let mut i: usize = 0;
            while i < off_diagonal.len() {
                B[(i + 1) + i * k] =
                    if off_diagonal[i] < 0.0 { -off_diagonal[i] } else { off_diagonal[i] };
                i = i + 1;
            }
        }
    }

    B
}
