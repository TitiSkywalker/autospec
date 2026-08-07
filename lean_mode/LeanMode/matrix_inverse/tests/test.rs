/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// Multiply two n×n matrices (row-major flat slices).
/// `c[i * n + j] = sum_k a[i * n + k] * b[k * n + j]`
fn matrix_mul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut c: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < n * n {
        c.push(0.0);
        idx += 1;
    }
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            let mut sum: f64 = 0.0;
            let mut k: usize = 0;
            while k < n {
                sum = sum + a[i * n + k] * b[k * n + j];
                k += 1;
            }
            c[i * n + j] = sum;
            j += 1;
        }
        i += 1;
    }
    c
}

/// Check that `m` is approximately the n×n identity matrix.
fn assert_identity(m: &[f64], n: usize, tol: f64) {
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_approx_eq(m[i * n + j], expected, tol);
            j += 1;
        }
        i += 1;
    }
}

/// Verify A * A_inv = I and A_inv * A = I.
fn verify_inverse(a: &[f64], a_inv: &[f64], n: usize) {
    let prod1 = matrix_mul(a, a_inv, n);
    assert_identity(&prod1, n, 1e-9);

    let prod2 = matrix_mul(a_inv, a, n);
    assert_identity(&prod2, n, 1e-9);
}

// ============================================================================
// Basic tests
// ============================================================================

/// n = 0: empty matrix, inverse is the empty matrix.
#[test]
fn test_inverse_empty() {
    let inv = matrix_inverse::matrix_inverse(&[], 0);
    assert!(inv.len() == 0);
}

/// n = 1: [[5]] → [[0.2]].
#[test]
fn test_inverse_1x1() {
    let a = [5.0];
    let inv = matrix_inverse::matrix_inverse(&a, 1);
    assert!(inv.len() == 1);
    assert_approx_eq(inv[0], 0.2, 1e-9);
    verify_inverse(&a, &inv, 1);
}

/// n = 1: [[-3]] → [[-1/3]].
#[test]
fn test_inverse_1x1_negative() {
    let a = [-3.0];
    let inv = matrix_inverse::matrix_inverse(&a, 1);
    assert!(inv.len() == 1);
    assert_approx_eq(inv[0], -1.0 / 3.0, 1e-9);
    verify_inverse(&a, &inv, 1);
}

/// n = 2: [[1, 2], [3, 4]].
/// det = -2, inverse = [[-2, 1], [1.5, -0.5]].
#[test]
fn test_inverse_2x2() {
    let a = [1.0, 2.0, 3.0, 4.0];
    let inv = matrix_inverse::matrix_inverse(&a, 2);
    assert!(inv.len() == 4);
    assert_approx_eq(inv[0], -2.0, 1e-9);
    assert_approx_eq(inv[1], 1.0, 1e-9);
    assert_approx_eq(inv[2], 1.5, 1e-9);
    assert_approx_eq(inv[3], -0.5, 1e-9);
    verify_inverse(&a, &inv, 2);
}

/// n = 2: identity → identity.
#[test]
fn test_inverse_2x2_identity() {
    let a = [1.0, 0.0, 0.0, 1.0];
    let inv = matrix_inverse::matrix_inverse(&a, 2);
    assert!(inv.len() == 4);
    verify_inverse(&a, &inv, 2);
}

/// n = 2: singular matrix [[1, 2], [2, 4]] → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_inverse_2x2_singular() {
    let a = [1.0, 2.0, 2.0, 4.0];
    let _ = matrix_inverse::matrix_inverse(&a, 2);
}

// ============================================================================
// 3×3 tests
// ============================================================================

/// n = 3: identity → identity.
#[test]
fn test_inverse_3x3_identity() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    ];
    let inv = matrix_inverse::matrix_inverse(&a, 3);
    assert!(inv.len() == 9);
    verify_inverse(&a, &inv, 3);
}

/// n = 3: [[2, 1, 1], [1, 2, 1], [1, 1, 2]].
/// det = 4, inverse = [[3/4, -1/4, -1/4], [-1/4, 3/4, -1/4], [-1/4, -1/4, 3/4]].
#[test]
fn test_inverse_3x3_symmetric() {
    #[rustfmt::skip]
    let a = [
        2.0, 1.0, 1.0,
        1.0, 2.0, 1.0,
        1.0, 1.0, 2.0,
    ];
    let inv = matrix_inverse::matrix_inverse(&a, 3);
    assert!(inv.len() == 9);
    assert_approx_eq(inv[0], 0.75, 1e-9);
    assert_approx_eq(inv[1], -0.25, 1e-9);
    assert_approx_eq(inv[2], -0.25, 1e-9);
    assert_approx_eq(inv[3], -0.25, 1e-9);
    assert_approx_eq(inv[4], 0.75, 1e-9);
    assert_approx_eq(inv[5], -0.25, 1e-9);
    assert_approx_eq(inv[6], -0.25, 1e-9);
    assert_approx_eq(inv[7], -0.25, 1e-9);
    assert_approx_eq(inv[8], 0.75, 1e-9);
    verify_inverse(&a, &inv, 3);
}

/// n = 3: general matrix [[1, 2, 3], [0, 1, 4], [5, 6, 0]].
/// det = 1, used in many textbook examples.
#[test]
fn test_inverse_3x3_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        0.0, 1.0, 4.0,
        5.0, 6.0, 0.0,
    ];
    let inv = matrix_inverse::matrix_inverse(&a, 3);
    assert!(inv.len() == 9);
    verify_inverse(&a, &inv, 3);
}

/// n = 3: singular (row of zeros) → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_inverse_3x3_singular() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        0.0, 0.0, 0.0,
    ];
    let _ = matrix_inverse::matrix_inverse(&a, 3);
}

/// n = 3: singular (linearly dependent rows) → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_inverse_3x3_singular2() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        2.0, 4.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let _ = matrix_inverse::matrix_inverse(&a, 3);
}

// ============================================================================
// 4×4 tests
// ============================================================================

/// n = 4: diagonal matrix [[1,0,0,0],[0,2,0,0],[0,0,3,0],[0,0,0,4]].
/// Inverse = [[1,0,0,0],[0,1/2,0,0],[0,0,1/3,0],[0,0,0,1/4]].
#[test]
fn test_inverse_4x4_diagonal() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 0.0, 0.0,
        0.0, 2.0, 0.0, 0.0,
        0.0, 0.0, 3.0, 0.0,
        0.0, 0.0, 0.0, 4.0,
    ];
    let inv = matrix_inverse::matrix_inverse(&a, 4);
    assert!(inv.len() == 16);
    verify_inverse(&a, &inv, 4);
}

/// n = 4: general invertible matrix.  Verify A * A⁻¹ = I.
#[test]
fn test_inverse_4x4_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 2.0, -1.0,
        3.0, 0.0, 0.0,  5.0,
        2.0, 1.0, 4.0, -3.0,
        1.0, 0.0, 5.0,  0.0,
    ];
    let inv = matrix_inverse::matrix_inverse(&a, 4);
    assert!(inv.len() == 16);
    verify_inverse(&a, &inv, 4);
}

/// n = 4: singular (row of zeros) → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_inverse_4x4_singular() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        0.0, 0.0, 0.0, 0.0,
        9.0, 10.0, 11.0, 12.0,
    ];
    let _ = matrix_inverse::matrix_inverse(&a, 4);
}
