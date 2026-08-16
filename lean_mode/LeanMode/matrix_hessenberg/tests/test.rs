use matrix_hessenberg::hessenberg_decomposition;

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

/// Transpose a square n×n column-major matrix.
fn transpose(a: &[f64], n: usize) -> Vec<f64> {
    let mut t = vec![0.0; n * n];
    for col in 0..n {
        for row in 0..n {
            t[row + col * n] = a[col + row * n];
        }
    }
    t
}

/// Verify A = Q · H · Qᵀ within tolerance.
fn verify_similarity(a: &[f64], q: &[f64], h: &[f64], n: usize, tol: f64) {
    let qt = transpose(q, n);
    let qh = mat_mul(q, h, n, n, n);
    let qhqt = mat_mul(&qh, &qt, n, n, n);
    for i in 0..n * n {
        assert_approx_eq(qhqt[i], a[i], tol);
    }
}

/// Verify Q is orthogonal: Qᵀ·Q = I and Q·Qᵀ = I.
fn verify_orthogonal(q: &[f64], n: usize, tol: f64) {
    let qt = transpose(q, n);
    let qtq = mat_mul(&qt, q, n, n, n);
    let qqt = mat_mul(q, &qt, n, n, n);
    for col in 0..n {
        for row in 0..n {
            let expected = if row == col { 1.0 } else { 0.0 };
            assert_approx_eq(qtq[row + col * n], expected, tol);
            assert_approx_eq(qqt[row + col * n], expected, tol);
        }
    }
}

/// Verify H is upper Hessenberg: H[i][j] = 0 for i > j + 1.
fn verify_hessenberg(h: &[f64], n: usize, tol: f64) {
    for col in 0..n {
        for row in (col + 2)..n {
            assert_approx_eq(h[row + col * n], 0.0, tol);
        }
    }
}

/// Full check: similarity, orthogonality, Hessenberg structure.
fn check_decomposition(a: &[f64], n: usize, tol: f64) {
    let result = hessenberg_decomposition(a, n);
    assert_eq!(result.q.len(), n * n);
    assert_eq!(result.h.len(), n * n);
    verify_similarity(a, &result.q, &result.h, n, tol);
    verify_orthogonal(&result.q, n, tol);
    verify_hessenberg(&result.h, n, tol);
}

// ============================================================================
// Basic tests
// ============================================================================

/// 0×0 empty matrix.
#[test]
fn test_hessenberg_empty() {
    let result = hessenberg_decomposition(&[], 0);
    assert!(result.q.is_empty());
    assert!(result.h.is_empty());
}

/// 1×1 matrix [[5]] → Q = [1], H = [5].
#[test]
fn test_hessenberg_1x1() {
    let a = [5.0];
    let result = hessenberg_decomposition(&a, 1);
    assert_eq!(result.q, vec![1.0]);
    assert_approx_eq(result.h[0], 5.0, 1e-12);
    check_decomposition(&a, 1, 1e-12);
}

/// 2×2 general [[1, 2], [3, 4]] (column-major [1, 3, 2, 4]).
#[test]
fn test_hessenberg_2x2_general() {
    let a = [1.0, 3.0, 2.0, 4.0];
    check_decomposition(&a, 2, 1e-12);
}

/// 2×2 identity.
#[test]
fn test_hessenberg_2x2_identity() {
    let a = [1.0, 0.0, 0.0, 1.0];
    check_decomposition(&a, 2, 1e-12);
}

/// 3×3 general matrix.
#[test]
fn test_hessenberg_3x3_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 10.0,
    ];
    check_decomposition(&a, 3, 1e-10);
}

/// 3×3 symmetric matrix.
#[test]
fn test_hessenberg_3x3_symmetric() {
    #[rustfmt::skip]
    let a = [
        4.0, 1.0, 2.0,
        1.0, 5.0, 3.0,
        2.0, 3.0, 6.0,
    ];
    check_decomposition(&a, 3, 1e-10);
}

/// 3×3 matrix that is already upper Hessenberg.
#[test]
fn test_hessenberg_3x3_already() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 0.0,
        5.0, 3.0, 4.0,
        0.0, 6.0, 7.0,
    ];
    check_decomposition(&a, 3, 1e-10);
}

/// 4×4 general matrix.
#[test]
fn test_hessenberg_4x4_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0,
    ];
    check_decomposition(&a, 4, 1e-9);
}

/// 4×4 with a zero column below the diagonal (tests the not_zero == false path).
#[test]
fn test_hessenberg_4x4_zero_subcolumn() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 3.0, 4.0,
        0.0, 5.0, 6.0, 7.0,
        0.0, 8.0, 9.0, 10.0,
        0.0, 11.0, 12.0, 13.0,
    ];
    check_decomposition(&a, 4, 1e-9);
}

/// 5×5 general matrix.
#[test]
fn test_hessenberg_5x5_general() {
    #[rustfmt::skip]
    let a = [
        2.0, 3.0, 5.0, 7.0, 11.0,
        13.0, 17.0, 19.0, 23.0, 29.0,
        31.0, 37.0, 41.0, 43.0, 47.0,
        53.0, 59.0, 61.0, 67.0, 71.0,
        73.0, 79.0, 83.0, 89.0, 97.0,
    ];
    check_decomposition(&a, 5, 1e-9);
}
