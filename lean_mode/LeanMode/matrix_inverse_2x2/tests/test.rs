/// Helper — assert two length-4 vectors are equal within a tolerance.
fn assert_vec4_approx_eq(actual: &[f64], expected: &[f64], tol: f64) {
    for i in 0..4 {
        let diff = actual[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: invert [[2, 1], [1, 1]]  →  det = 1,  inv = [[1, -1], [-1, 2]]
#[test]
fn test_invert_simple() {
    let m = vec![2.0, 1.0, 1.0, 1.0];
    let expected = vec![1.0, -1.0, -1.0, 2.0];
    let result = matrix_inverse_2x2::invert_2x2(&m);
    assert_vec4_approx_eq(&result, &expected, 0.001);
}

/// Test: identity matrix → inverse is identity
#[test]
fn test_invert_identity() {
    let m = vec![1.0, 0.0, 0.0, 1.0];
    let expected = vec![1.0, 0.0, 0.0, 1.0];
    let result = matrix_inverse_2x2::invert_2x2(&m);
    assert_vec4_approx_eq(&result, &expected, 0.001);
}

/// Test: singular matrix (det = 0) → should panic
#[test]
#[should_panic(expected = "singular matrix")]
fn test_invert_singular() {
    let m = vec![1.0, 2.0, 2.0, 4.0]; // rows are linearly dependent
    matrix_inverse_2x2::invert_2x2(&m);
}

/// Test: diagonal matrix → element-wise reciprocals
#[test]
fn test_invert_diagonal() {
    let m = vec![3.0, 0.0, 0.0, 5.0];
    let expected = vec![1.0 / 3.0, 0.0, 0.0, 1.0 / 5.0];
    let result = matrix_inverse_2x2::invert_2x2(&m);
    assert_vec4_approx_eq(&result, &expected, 0.001);
}

/// Test: matrix with a positive determinant
#[test]
fn test_invert_positive_det() {
    // [[4, 7], [2, 6]] → det = 24-14 = 10
    let m = vec![4.0, 7.0, 2.0, 6.0];
    let expected = vec![0.6, -0.7, -0.2, 0.4];
    let result = matrix_inverse_2x2::invert_2x2(&m);
    assert_vec4_approx_eq(&result, &expected, 0.001);
}

/// Test: matrix with large entries
#[test]
fn test_invert_large() {
    // [[100, 0], [0, 200]] → det = 20000
    let m = vec![100.0, 0.0, 0.0, 200.0];
    let expected = vec![0.01, 0.0, 0.0, 0.005];
    let result = matrix_inverse_2x2::invert_2x2(&m);
    assert_vec4_approx_eq(&result, &expected, 0.001);
}
