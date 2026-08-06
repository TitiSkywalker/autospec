/// Helper — assert two f64 values are equal within a tolerance.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = actual - expected;
    assert!(diff.abs() <= tol);
}

/// Test: identity matrix → det = 1
#[test]
fn test_det_identity() {
    let m = vec![1.0, 0.0, 0.0, 1.0];
    assert_approx_eq(matrix_det_2x2::det_2x2(&m), 1.0, 0.001);
}

/// Test: simple matrix → det = a*d - b*c
#[test]
fn test_det_simple() {
    // [[2, 1], [1, 1]] → det = 2*1 - 1*1 = 1
    let m = vec![2.0, 1.0, 1.0, 1.0];
    assert_approx_eq(matrix_det_2x2::det_2x2(&m), 1.0, 0.001);
}

/// Test: singular matrix → det = 0
#[test]
fn test_det_singular() {
    // [[1, 2], [2, 4]] → det = 1*4 - 2*2 = 0
    let m = vec![1.0, 2.0, 2.0, 4.0];
    assert_approx_eq(matrix_det_2x2::det_2x2(&m), 0.0, 0.001);
}

/// Test: diagonal matrix
#[test]
fn test_det_diagonal() {
    // [[3, 0], [0, 5]] → det = 3*5 - 0 = 15
    let m = vec![3.0, 0.0, 0.0, 5.0];
    assert_approx_eq(matrix_det_2x2::det_2x2(&m), 15.0, 0.001);
}

/// Test: matrix with negative entries
#[test]
fn test_det_negative() {
    // [[4, 7], [2, 6]] → det = 4*6 - 7*2 = 24-14 = 10
    let m = vec![4.0, 7.0, 2.0, 6.0];
    assert_approx_eq(matrix_det_2x2::det_2x2(&m), 10.0, 0.001);
}

/// Test: zero matrix → det = 0
#[test]
fn test_det_zero() {
    let m = vec![0.0, 0.0, 0.0, 0.0];
    assert_approx_eq(matrix_det_2x2::det_2x2(&m), 0.0, 0.001);
}
