/// Helper — compare two f64 values within tolerance.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// n = 0: empty matrix → characteristic polynomial 1.
#[test]
fn test_charpoly_empty() {
    let c = matrix_charpoly::charpoly(&[], 0);
    assert_eq!(c.len(), 1);
    assert_approx_eq(c[0], 1.0, 0.001);
}

/// n = 1: [[5]] → x − 5.
#[test]
fn test_charpoly_1x1() {
    let c = matrix_charpoly::charpoly(&[5.0], 1);
    assert_approx_eq(c[0], -5.0, 0.001);
    assert_approx_eq(c[1], 1.0, 0.001);
}

/// n = 1: [[-3]] → x + 3.
#[test]
fn test_charpoly_1x1_negative() {
    let c = matrix_charpoly::charpoly(&[-3.0], 1);
    assert_approx_eq(c[0], 3.0, 0.001);
    assert_approx_eq(c[1], 1.0, 0.001);
}

/// n = 2: [[1, 2], [3, 4]] → x² − 5x − 2.
#[test]
fn test_charpoly_2x2() {
    let m = vec![1.0, 2.0, 3.0, 4.0];
    let c = matrix_charpoly::charpoly(&m, 2);
    assert_approx_eq(c[0], -2.0, 0.001);
    assert_approx_eq(c[1], -5.0, 0.001);
    assert_approx_eq(c[2], 1.0, 0.001);
}

/// n = 2: identity → (x − 1)² = x² − 2x + 1.
#[test]
fn test_charpoly_2x2_identity() {
    let m = vec![1.0, 0.0, 0.0, 1.0];
    let c = matrix_charpoly::charpoly(&m, 2);
    assert_approx_eq(c[0], 1.0, 0.001);
    assert_approx_eq(c[1], -2.0, 0.001);
    assert_approx_eq(c[2], 1.0, 0.001);
}

/// n = 3: diagonal 2, 3, 4 → (x−2)(x−3)(x−4) = x³ − 9x² + 26x − 24.
#[test]
fn test_charpoly_3x3_diagonal() {
    #[rustfmt::skip]
    let m = vec![
        2.0, 0.0, 0.0,
        0.0, 3.0, 0.0,
        0.0, 0.0, 4.0,
    ];
    let c = matrix_charpoly::charpoly(&m, 3);
    assert_approx_eq(c[0], -24.0, 0.001);
    assert_approx_eq(c[1], 26.0, 0.001);
    assert_approx_eq(c[2], -9.0, 0.001);
    assert_approx_eq(c[3], 1.0, 0.001);
}

/// n = 3: [[1, 2, 3], [4, 5, 6], [7, 8, 10]] → x³ − 16x² − 12x + 3.
#[test]
fn test_charpoly_3x3_general() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 10.0,
    ];
    let c = matrix_charpoly::charpoly(&m, 3);
    assert_approx_eq(c[0], 3.0, 0.001);
    assert_approx_eq(c[1], -12.0, 0.001);
    assert_approx_eq(c[2], -16.0, 0.001);
    assert_approx_eq(c[3], 1.0, 0.001);
}
