use matrix_bidiagonal::bidiagonal_decomposition;

/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol, "{} != {} (diff {})", actual, expected, diff);
}

/// Multiply two matrices (column-major, A is a_rows × a_cols, B is a_cols × b_cols).
fn mat_mul(a: &[f64], b: &[f64], a_rows: usize, a_cols: usize, b_cols: usize) -> Vec<f64> {
    let mut result = vec![0.0; a_rows * b_cols];
    for col in 0..b_cols {
        for row in 0..a_rows {
            let mut sum = 0.0;
            for k in 0..a_cols {
                sum += a[row + k * a_rows] * b[k + col * a_cols];
            }
            result[row + col * a_rows] = sum;
        }
    }
    result
}

/// Transpose a matrix (column-major, `rows` × `cols` → `cols` × `rows`).
fn transpose(a: &[f64], rows: usize, cols: usize) -> Vec<f64> {
    let mut t = vec![0.0; rows * cols];
    for col in 0..cols {
        for row in 0..rows {
            t[col + row * cols] = a[row + col * rows];
        }
    }
    t
}

/// Verify A = U · B · Vᵀ within tolerance.
fn verify_similarity(a: &[f64], u: &[f64], b: &[f64], v_t: &[f64], m: usize, n: usize, tol: f64) {
    let ub = mat_mul(u, b, m, n, n);
    let ubvt = mat_mul(&ub, v_t, m, n, n);
    for i in 0..m * n {
        assert_approx_eq(ubvt[i], a[i], tol);
    }
}

/// Verify U has orthonormal columns: Uᵀ·U = I_n.
fn verify_u_orthonormal(u: &[f64], m: usize, n: usize, tol: f64) {
    let ut = transpose(u, m, n); // n × m
    let utu = mat_mul(&ut, u, n, m, n); // n × n
    for col in 0..n {
        for row in 0..n {
            let expected = if row == col { 1.0 } else { 0.0 };
            assert_approx_eq(utu[row + col * n], expected, tol);
        }
    }
}

/// Verify Vᵀ has orthonormal rows: Vᵀ·V = I_n.
fn verify_vt_orthonormal(v_t: &[f64], n: usize, tol: f64) {
    let v = transpose(v_t, n, n); // V = (Vᵀ)ᵀ, n × n
    let vtv = mat_mul(v_t, &v, n, n, n); // (n×n)·(n×n) = n×n
    for col in 0..n {
        for row in 0..n {
            let expected = if row == col { 1.0 } else { 0.0 };
            assert_approx_eq(vtv[row + col * n], expected, tol);
        }
    }
}

/// Verify B is upper bidiagonal: zero below the diagonal (i > j) and above
/// the first superdiagonal (i + 1 < j).
fn verify_bidiagonal(b: &[f64], n: usize, tol: f64) {
    for col in 0..n {
        for row in 0..n {
            let below_diag = row > col;
            let above_super = row + 1 < col;
            if below_diag || above_super {
                assert_approx_eq(b[row + col * n], 0.0, tol);
            }
        }
    }
}

/// Full check: similarity, orthonormality, bidiagonal structure.
fn check_decomposition(a: &[f64], m: usize, n: usize, tol: f64) {
    let result = bidiagonal_decomposition(a, m, n);
    assert_eq!(result.u.len(), m * n);
    assert_eq!(result.b.len(), n * n);
    assert_eq!(result.v_t.len(), n * n);
    verify_similarity(a, &result.u, &result.b, &result.v_t, m, n, tol);
    verify_u_orthonormal(&result.u, m, n, tol);
    verify_vt_orthonormal(&result.v_t, n, tol);
    verify_bidiagonal(&result.b, n, tol);
}

// ============================================================================
// Basic tests
// ============================================================================

/// 1×1 matrix [[5]] → U = [1], B = [5], Vᵀ = [1].
#[test]
fn test_bidiagonal_1x1() {
    let a = [5.0];
    let result = bidiagonal_decomposition(&a, 1, 1);
    assert_eq!(result.u, vec![1.0]);
    assert_approx_eq(result.b[0], 5.0, 1e-12);
    assert_eq!(result.v_t, vec![1.0]);
    check_decomposition(&a, 1, 1, 1e-12);
}

/// 2×2 general [[1, 2], [3, 4]] (column-major [1, 3, 2, 4]).
#[test]
fn test_bidiagonal_2x2_general() {
    let a = [1.0, 3.0, 2.0, 4.0];
    check_decomposition(&a, 2, 2, 1e-12);
}

/// 2×2 identity.
#[test]
fn test_bidiagonal_2x2_identity() {
    let a = [1.0, 0.0, 0.0, 1.0];
    check_decomposition(&a, 2, 2, 1e-12);
}

/// 3×3 general matrix.
#[test]
fn test_bidiagonal_3x3_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 10.0,
    ];
    check_decomposition(&a, 3, 3, 1e-10);
}

/// 3×3 symmetric matrix.
#[test]
fn test_bidiagonal_3x3_symmetric() {
    #[rustfmt::skip]
    let a = [
        4.0, 1.0, 2.0,
        1.0, 5.0, 3.0,
        2.0, 3.0, 6.0,
    ];
    check_decomposition(&a, 3, 3, 1e-10);
}

/// 3×3 matrix with a zero row segment (exercises the zero-axis skip path).
#[test]
fn test_bidiagonal_3x3_zero_segment() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        0.0, 4.0, 5.0,
        0.0, 0.0, 6.0,
    ];
    check_decomposition(&a, 3, 3, 1e-10);
}

/// 4×4 general matrix.
#[test]
fn test_bidiagonal_4x4_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0,
    ];
    check_decomposition(&a, 4, 4, 1e-9);
}

/// 5×5 general matrix.
#[test]
fn test_bidiagonal_5x5_general() {
    #[rustfmt::skip]
    let a = [
        2.0, 3.0, 5.0, 7.0, 11.0,
        13.0, 17.0, 19.0, 23.0, 29.0,
        31.0, 37.0, 41.0, 43.0, 47.0,
        53.0, 59.0, 61.0, 67.0, 71.0,
        73.0, 79.0, 83.0, 89.0, 97.0,
    ];
    check_decomposition(&a, 5, 5, 1e-9);
}

// ============================================================================
// Tall (rectangular, nrows > ncols) matrices
// ============================================================================

/// 3×2 tall matrix.
#[test]
fn test_bidiagonal_3x2_tall() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0,
        3.0, 4.0,
        5.0, 6.0,
    ];
    check_decomposition(&a, 3, 2, 1e-10);
}

/// 4×2 tall matrix.
#[test]
fn test_bidiagonal_4x2_tall() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0,
        3.0, 4.0,
        5.0, 6.0,
        7.0, 8.0,
    ];
    check_decomposition(&a, 4, 2, 1e-10);
}

/// 5×3 tall matrix.
#[test]
fn test_bidiagonal_5x3_tall() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
        10.0, 11.0, 12.0,
        13.0, 14.0, 15.0,
    ];
    check_decomposition(&a, 5, 3, 1e-9);
}
