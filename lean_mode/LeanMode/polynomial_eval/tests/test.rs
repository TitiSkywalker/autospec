/// Test: original example — 4x² + 2x - 8 at x = 3
#[test]
fn test_known_answer_original() {
    // coeffs = [-8, 2, 4] → -8 + 2x + 4x²
    // At x=3: -8 + 6 + 36 = 34
    let coeffs = [-8.0, 2.0, 4.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 3.0), 34.0);
}

/// Test: all-ones polynomial at x = 1
#[test]
fn test_known_answer_all_ones() {
    // 1 + x + x² + x³ at x=1 = 4
    let coeffs = [1.0, 1.0, 1.0, 1.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 1.0), 4.0);
}

/// Test: negative coefficients at x = 1
#[test]
fn test_known_answer_negative_coeffs() {
    // -2 - 2x - 2x² - 2x³ at x=1 = -8
    let coeffs = [-2.0, -2.0, -2.0, -2.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 1.0), -8.0);
}

/// Test: constant polynomial
#[test]
fn test_constant_only() {
    // 100 + 0x + 0x² + 0x³ at x=9 = 100
    let coeffs = [100.0, 0.0, 0.0, 0.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 9.0), 100.0);
}

/// Test: linear only
#[test]
fn test_linear_only() {
    // 0 + 1x + 0x² + 0x³ at x=9 = 9
    let coeffs = [0.0, 1.0, 0.0, 0.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 9.0), 9.0);
}

/// Test: quadratic
#[test]
fn test_quadratic() {
    // 0 + 0 - 1x² + 0x³ at x=9 = -81
    let coeffs = [0.0, 0.0, -1.0, 0.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 9.0), -81.0);
}

/// Test: cubic with mixed signs
#[test]
fn test_cubic_mixed() {
    // 0 + (-9)x + 0x² + 40x³ at x=2
    // = -18 + 320 = 302
    let coeffs = [0.0, -9.0, 0.0, 40.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 2.0), 302.0);
}

/// Test: evaluation at x = 0 gives constant term
#[test]
fn test_property_eval_at_zero() {
    let coeffs = [5.0, 3.0, 2.0, 1.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 0.0), 5.0);
    let coeffs2 = [-3.0, 0.0, 0.0, 1.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs2, 0.0), -3.0);
}

/// Test: linearity — eval(a + bx) = a + bx
#[test]
fn test_property_linearity() {
    let a: f64 = 5.0;
    let b: f64 = 3.0;
    let x: f64 = 7.0;
    let coeffs = [a, b];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, x), a + b * x);
}

/// Test: quadratic form — eval(a + bx + cx²) = a + bx + cx²
#[test]
fn test_property_quadratic_form() {
    let a: f64 = 2.0;
    let b: f64 = -3.0;
    let c: f64 = 4.0;
    let x: f64 = 5.0;
    let coeffs = [a, b, c];
    assert_eq!(
        polynomial_eval::polynomial_eval(&coeffs, x),
        a + b * x + c * x * x
    );
}

/// Test: edge case — single coefficient
#[test]
fn test_edge_case_single_coefficient() {
    let coeffs = [42.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 100.0), 42.0);
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, -100.0), 42.0);
}

/// Test: edge case — zero polynomial
#[test]
fn test_edge_case_zero_polynomial() {
    let coeffs = [0.0, 0.0, 0.0, 0.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 5.0), 0.0);
}

/// Test: edge case — negative x
#[test]
fn test_edge_case_negative_x() {
    // 1 + 2x + 3x² at x=-2: 1 - 4 + 12 = 9
    let coeffs = [1.0, 2.0, 3.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, -2.0), 9.0);
}

/// Test: float with fractional coefficients
#[test]
fn test_float_fractional() {
    // 1 + 2x + 3x² at x=2.5: 1 + 5 + 18.75 = 24.75
    let coeffs = [1.0, 2.0, 3.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 2.5), 24.75);
}

/// Test: floating point with negative x
#[test]
fn test_float_negative_x() {
    // 1 - 2x + 3x² at x=1.5: 1 - 3 + 6.75 = 4.75
    let coeffs = [1.0, -2.0, 3.0];
    assert_eq!(polynomial_eval::polynomial_eval(&coeffs, 1.5), 4.75);
}
