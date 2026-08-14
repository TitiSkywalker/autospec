use matrix_pow::matrix_pow;

/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// Naive n×n matrix multiply (row-major) for reference comparison.
fn reference_mul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut c = vec![0.0; n * n];
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

/// A^0 = I.
#[test]
fn test_pow_zero_identity() {
    let a = [1.0, 2.0, 3.0, 4.0];
    let result = matrix_pow(&a, 2, 0);
    assert_approx_eq(result[0], 1.0, 1e-9);
    assert_approx_eq(result[1], 0.0, 1e-9);
    assert_approx_eq(result[2], 0.0, 1e-9);
    assert_approx_eq(result[3], 1.0, 1e-9);
}

/// A^1 = A.
#[test]
fn test_pow_one() {
    let a = [1.0, 2.0, 3.0, 4.0];
    let result = matrix_pow(&a, 2, 1);
    let mut i: usize = 0;
    while i < 4 {
        assert_approx_eq(result[i], a[i], 1e-9);
        i += 1;
    }
}

/// A^2 for [[1, 2], [3, 4]] = [[7, 10], [15, 22]].
#[test]
fn test_pow_two() {
    let a = [1.0, 2.0, 3.0, 4.0];
    let result = matrix_pow(&a, 2, 2);
    let expected = [7.0, 10.0, 15.0, 22.0];
    let mut i: usize = 0;
    while i < 4 {
        assert_approx_eq(result[i], expected[i], 1e-9);
        i += 1;
    }
}

/// A^3 for [[1, 2], [3, 4]] = [[37, 54], [81, 118]].
#[test]
fn test_pow_three() {
    let a = [1.0, 2.0, 3.0, 4.0];
    let result = matrix_pow(&a, 2, 3);
    let expected = [37.0, 54.0, 81.0, 118.0];
    let mut i: usize = 0;
    while i < 4 {
        assert_approx_eq(result[i], expected[i], 1e-9);
        i += 1;
    }
}

/// A^5 matches naive repeated multiplication for a 3×3 matrix.
#[test]
fn test_pow_matches_naive() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 2.0,
        0.0, 3.0, 0.0,
        1.0, 0.0, 1.0,
    ];
    let result = matrix_pow(&a, 3, 5);

    // Naive reference: repeated squaring by hand.
    let mut expected = a.to_vec();
    let mut e: usize = 1;
    while e < 5 {
        expected = reference_mul(&expected, &a, 3);
        e += 1;
    }

    let mut i: usize = 0;
    while i < 9 {
        assert_approx_eq(result[i], expected[i], 1e-9);
        i += 1;
    }
}

/// 1×1 matrix: [x]^e = [x^e].
#[test]
fn test_pow_1x1() {
    let a = [3.0];
    let result = matrix_pow(&a, 1, 4);
    assert_approx_eq(result[0], 81.0, 1e-9);
}
