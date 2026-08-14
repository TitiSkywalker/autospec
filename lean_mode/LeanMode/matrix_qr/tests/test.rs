use matrix_qr::qr_decomposition;

/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol, "{} != {} (diff {})", actual, expected, diff);
}

/// Multiply two matrices (column-major, A is a_rows × a_cols, B is a_cols × b_cols).
fn matrix_mul(a: &[f64], b: &[f64], a_rows: usize, a_cols: usize, b_cols: usize) -> Vec<f64> {
    let mut result = vec![0.0; a_rows * b_cols];
    let mut col: usize = 0;
    while col < b_cols {
        let mut row: usize = 0;
        while row < a_rows {
            let mut sum: f64 = 0.0;
            let mut k: usize = 0;
            while k < a_cols {
                sum = sum + a[row + k * a_rows] * b[k + col * a_cols];
                k += 1;
            }
            result[row + col * a_rows] = sum;
            row += 1;
        }
        col += 1;
    }
    result
}

/// Verify A = Q · R within tolerance.
fn verify_qr(a: &[f64], q: &[f64], r: &[f64], nrows: usize, ncols: usize, tol: f64) {
    let min_dim = if nrows < ncols { nrows } else { ncols };
    let product = matrix_mul(q, r, nrows, min_dim, ncols);
    let mut i: usize = 0;
    while i < nrows * ncols {
        assert_approx_eq(product[i], a[i], tol);
        i += 1;
    }
}

// ============================================================================
// Basic tests
// ============================================================================

/// 0×0 empty matrix.
#[test]
fn test_qr_empty() {
    let result = qr_decomposition(&[], 0, 0);
    assert!(result.q.is_empty());
    assert!(result.r.is_empty());
}

/// 1×1 matrix [[5]] → Q = [1], R = [5].
#[test]
fn test_qr_1x1() {
    let a = [5.0];
    let result = qr_decomposition(&a, 1, 1);
    assert_eq!(result.q.len(), 1);
    assert_eq!(result.r.len(), 1);
    assert_approx_eq(result.r[0], 5.0, 1e-9);
    verify_qr(&a, &result.q, &result.r, 1, 1, 1e-9);
}

/// 2×2 identity.
#[test]
fn test_qr_2x2_identity() {
    let a = [1.0, 0.0, 0.0, 1.0];
    let result = qr_decomposition(&a, 2, 2);
    verify_qr(&a, &result.q, &result.r, 2, 2, 1e-9);
}

/// 2×2 general [[1, 2], [3, 4]] (column-major [1, 3, 2, 4]).
#[test]
fn test_qr_2x2_general() {
    let a = [1.0, 3.0, 2.0, 4.0];
    let result = qr_decomposition(&a, 2, 2);
    verify_qr(&a, &result.q, &result.r, 2, 2, 1e-9);

    // R is upper triangular: R[1,0] = 0.
    assert_approx_eq(result.r[1], 0.0, 1e-9);
}

/// 3×2 tall matrix [[1, 4], [2, 5], [3, 6]] (column-major [1,2,3,4,5,6]).
#[test]
fn test_qr_3x2() {
    let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let result = qr_decomposition(&a, 3, 2);
    verify_qr(&a, &result.q, &result.r, 3, 2, 1e-9);

    assert_approx_eq(result.r[1], 0.0, 1e-9); // R[1,0] = 0
}

/// 4×3 rectangular matrix.
#[test]
fn test_qr_4x3() {
    #[rustfmt::skip]
    let a = [
        1.0, 5.0, 9.0,
        2.0, 6.0, 10.0,
        3.0, 7.0, 11.0,
        4.0, 8.0, 12.0,
    ];
    let result = qr_decomposition(&a, 4, 3);
    verify_qr(&a, &result.q, &result.r, 4, 3, 1e-9);
}

/// Single column matrix (4×1).
#[test]
fn test_qr_single_column() {
    let a = [1.0, 2.0, 3.0, 4.0];
    let result = qr_decomposition(&a, 4, 1);
    verify_qr(&a, &result.q, &result.r, 4, 1, 1e-9);
}

// ============================================================================
// Orthogonality / structure
// ============================================================================

/// Q has orthonormal columns: Qᵀ · Q = I.
#[test]
fn test_qr_orthogonal() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 10.0,
    ];
    let result = qr_decomposition(&a, 3, 3);
    let q = &result.q;

    // (Qᵀ·Q)[i][j] = Σ_k Q[k][i] · Q[k][j] = δ_ij.
    let mut i: usize = 0;
    while i < 3 {
        let mut j: usize = 0;
        while j < 3 {
            let mut sum: f64 = 0.0;
            let mut k: usize = 0;
            while k < 3 {
                sum = sum + q[k + i * 3] * q[k + j * 3];
                k += 1;
            }
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_approx_eq(sum, expected, 1e-9);
            j += 1;
        }
        i += 1;
    }
}

/// R is upper triangular.
#[test]
fn test_qr_upper_triangular() {
    #[rustfmt::skip]
    let a = [
        3.0, 1.0, 4.0,
        1.0, 5.0, 9.0,
        2.0, 6.0, 5.0,
    ];
    let result = qr_decomposition(&a, 3, 3);
    let r = &result.r;
    let mut col: usize = 0;
    while col < 3 {
        let mut row: usize = col + 1;
        while row < 3 {
            assert_approx_eq(r[row + col * 3], 0.0, 1e-9);
            row += 1;
        }
        col += 1;
    }
}
