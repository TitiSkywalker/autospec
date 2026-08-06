/// Helper — compare two f64 values within tolerance.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// n = 0: empty matrix, determinant is 1 by convention.
#[test]
fn test_det_empty() {
    assert_approx_eq(matrix_det::det(&[], 0), 1.0, 0.001);
}

/// n = 1: [[5]] → 5.
#[test]
fn test_det_1x1() {
    assert_approx_eq(matrix_det::det(&[5.0], 1), 5.0, 0.001);
}

/// n = 1: [[-3]] → -3.
#[test]
fn test_det_1x1_negative() {
    assert_approx_eq(matrix_det::det(&[-3.0], 1), -3.0, 0.001);
}

/// n = 2: [[1, 2], [3, 4]] → 1*4 - 2*3 = -2.
#[test]
fn test_det_2x2() {
    let m = vec![1.0, 2.0, 3.0, 4.0];
    assert_approx_eq(matrix_det::det(&m, 2), -2.0, 0.001);
}

/// n = 2: identity → 1.
#[test]
fn test_det_2x2_identity() {
    let m = vec![1.0, 0.0, 0.0, 1.0];
    assert_approx_eq(matrix_det::det(&m, 2), 1.0, 0.001);
}

/// n = 3: identity → 1.
#[test]
fn test_det_3x3_identity() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    ];
    assert_approx_eq(matrix_det::det(&m, 3), 1.0, 0.001);
}

/// n = 3: diagonal → 2*3*4 = 24.
#[test]
fn test_det_3x3_diagonal() {
    #[rustfmt::skip]
    let m = vec![
        2.0, 0.0, 0.0,
        0.0, 3.0, 0.0,
        0.0, 0.0, 4.0,
    ];
    assert_approx_eq(matrix_det::det(&m, 3), 24.0, 0.001);
}

/// n = 3: general matrix.
/// [[1, 2, 3], [4, 5, 6], [7, 8, 10]] → det = -3.
/// (last element is 10 instead of 9 to make it nonsingular)
#[test]
fn test_det_3x3_general() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 10.0,
    ];
    assert_approx_eq(matrix_det::det(&m, 3), -3.0, 0.001);
}

/// n = 3: singular (two identical rows) → 0.
#[test]
fn test_det_3x3_singular() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        1.0, 2.0, 3.0,
    ];
    assert_approx_eq(matrix_det::det(&m, 3), 0.0, 0.001);
}

/// n = 3: singular (linearly dependent rows) → 0.
#[test]
fn test_det_3x3_singular2() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        2.0, 4.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    assert_approx_eq(matrix_det::det(&m, 3), 0.0, 0.001);
}

/// n = 4: known determinant.
/// [[1, 0, 2, -1], [3, 0, 0, 5], [2, 1, 4, -3], [1, 0, 5, 0]]
/// det = 30.
#[test]
fn test_det_4x4() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 0.0, 2.0, -1.0,
        3.0, 0.0, 0.0,  5.0,
        2.0, 1.0, 4.0, -3.0,
        1.0, 0.0, 5.0,  0.0,
    ];
    assert_approx_eq(matrix_det::det(&m, 4), 30.0, 0.001);
}
