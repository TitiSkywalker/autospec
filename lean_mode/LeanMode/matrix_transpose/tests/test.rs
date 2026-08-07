/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// Helper — check two matrices are approximately equal.
fn assert_matrix_eq(a: &[f64], b: &[f64], n: usize, tol: f64) {
    let mut i: usize = 0;
    while i < n * n {
        assert_approx_eq(a[i], b[i], tol);
        i += 1;
    }
}

// ============================================================================
// Basic tests
// ============================================================================

/// n = 0: empty matrix.
#[test]
fn test_transpose_empty() {
    let t = matrix_transpose::transpose(&[], 0);
    assert!(t.len() == 0);
}

/// n = 1: [[5]]^T = [[5]].
#[test]
fn test_transpose_1x1() {
    let t = matrix_transpose::transpose(&[5.0], 1);
    assert!(t.len() == 1);
    assert_approx_eq(t[0], 5.0, 1e-9);
}

/// n = 2: [[1, 2], [3, 4]]^T = [[1, 3], [2, 4]].
#[test]
fn test_transpose_2x2() {
    let a = [1.0, 2.0, 3.0, 4.0];
    let t = matrix_transpose::transpose(&a, 2);
    let expected = [1.0, 3.0, 2.0, 4.0];
    assert_matrix_eq(&t, &expected, 2, 1e-9);
}

/// n = 2: identity^T = identity.
#[test]
fn test_transpose_2x2_identity() {
    let a = [1.0, 0.0, 0.0, 1.0];
    let t = matrix_transpose::transpose(&a, 2);
    assert_matrix_eq(&t, &a, 2, 1e-9);
}

/// n = 2: symmetric matrix → transpose equals original.
#[test]
fn test_transpose_2x2_symmetric() {
    let a = [1.0, 2.0, 2.0, 3.0];
    let t = matrix_transpose::transpose(&a, 2);
    assert_matrix_eq(&t, &a, 2, 1e-9);
}

// ============================================================================
// 3×3 tests
// ============================================================================

/// n = 3: identity^T = identity.
#[test]
fn test_transpose_3x3_identity() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    ];
    let t = matrix_transpose::transpose(&a, 3);
    assert_matrix_eq(&t, &a, 3, 1e-9);
}

/// n = 3: general matrix.
/// [[1, 2, 3], [4, 5, 6], [7, 8, 9]]^T = [[1, 4, 7], [2, 5, 8], [3, 6, 9]].
#[test]
fn test_transpose_3x3_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    #[rustfmt::skip]
    let expected = [
        1.0, 4.0, 7.0,
        2.0, 5.0, 8.0,
        3.0, 6.0, 9.0,
    ];
    let t = matrix_transpose::transpose(&a, 3);
    assert_matrix_eq(&t, &expected, 3, 1e-9);
}

/// n = 3: symmetric matrix → transpose equals original.
#[test]
fn test_transpose_3x3_symmetric() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        2.0, 4.0, 5.0,
        3.0, 5.0, 6.0,
    ];
    let t = matrix_transpose::transpose(&a, 3);
    assert_matrix_eq(&t, &a, 3, 1e-9);
}

/// n = 3: upper triangular → transpose is lower triangular.
#[test]
fn test_transpose_3x3_triangular() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        0.0, 4.0, 5.0,
        0.0, 0.0, 6.0,
    ];
    #[rustfmt::skip]
    let expected = [
        1.0, 0.0, 0.0,
        2.0, 4.0, 0.0,
        3.0, 5.0, 6.0,
    ];
    let t = matrix_transpose::transpose(&a, 3);
    assert_matrix_eq(&t, &expected, 3, 1e-9);
}

// ============================================================================
// Involution property: (A^T)^T = A
// ============================================================================

/// n = 3: double transpose equals original.
#[test]
fn test_transpose_involution() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let t1 = matrix_transpose::transpose(&a, 3);
    let t2 = matrix_transpose::transpose(&t1, 3);
    assert_matrix_eq(&t2, &a, 3, 1e-9);
}

// ============================================================================
// 4×4 tests
// ============================================================================

/// n = 4: general matrix.
#[test]
fn test_transpose_4x4() {
    #[rustfmt::skip]
    let a = [
        1.0,  2.0,  3.0,  4.0,
        5.0,  6.0,  7.0,  8.0,
        9.0, 10.0, 11.0, 12.0,
       13.0, 14.0, 15.0, 16.0,
    ];
    #[rustfmt::skip]
    let expected = [
        1.0, 5.0,  9.0, 13.0,
        2.0, 6.0, 10.0, 14.0,
        3.0, 7.0, 11.0, 15.0,
        4.0, 8.0, 12.0, 16.0,
    ];
    let t = matrix_transpose::transpose(&a, 4);
    assert_matrix_eq(&t, &expected, 4, 1e-9);
}
