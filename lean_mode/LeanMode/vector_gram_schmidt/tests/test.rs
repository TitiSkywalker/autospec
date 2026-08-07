use vector_gram_schmidt::gram_schmidt_step;

/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// Compute the dot product of two vectors of length n.
fn dot_product(a: &[f64], b: &[f64], n: usize) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: usize = 0;
    while i < n {
        sum = sum + a[i] * b[i];
        i += 1;
    }
    sum
}

/// Verify that `result` is orthogonal to `u` (⟨result, u⟩ ≈ 0).
fn verify_orthogonal(result: &[f64], u: &[f64], n: usize, tol: f64) {
    let dot = dot_product(result, u, n);
    assert_approx_eq(dot, 0.0, tol);
}

/// Verify that `result = v - factor * u` where factor = ⟨v,u⟩ / ⟨u,u⟩.
fn verify_gs_step(result: &[f64], u: &[f64], v: &[f64], n: usize, tol: f64) {
    let dot_uu = dot_product(u, u, n);
    let dot_vu = dot_product(v, u, n);
    let factor = dot_vu / dot_uu;
    let mut i: usize = 0;
    while i < n {
        let expected = v[i] - factor * u[i];
        assert_approx_eq(result[i], expected, tol);
        i += 1;
    }
}

// ============================================================================
// Basic tests
// ============================================================================

/// n = 0: empty vectors.
#[test]
fn test_gs_empty() {
    let result = gram_schmidt_step(&[], &[], 0);
    assert!(result.len() == 0);
}

/// Zero u vector → panic.
#[test]
#[should_panic(expected = "zero vector")]
fn test_gs_zero_u() {
    let u = [0.0, 0.0];
    let v = [1.0, 2.0];
    let _ = gram_schmidt_step(&u, &v, 2);
}

/// u = [1, 0], v = [0, 1] — already orthogonal, result = v.
#[test]
fn test_gs_already_orthogonal() {
    let u = [1.0, 0.0];
    let v = [0.0, 1.0];
    let result = gram_schmidt_step(&u, &v, 2);
    assert_approx_eq(result[0], 0.0, 1e-9);
    assert_approx_eq(result[1], 1.0, 1e-9);
    verify_orthogonal(&result, &u, 2, 1e-9);
}

/// u = [2, 0], v = [2, 0] — identical vectors, result = [0, 0].
#[test]
fn test_gs_identical() {
    let u = [2.0, 0.0];
    let v = [2.0, 0.0];
    let result = gram_schmidt_step(&u, &v, 2);
    assert_approx_eq(result[0], 0.0, 1e-9);
    assert_approx_eq(result[1], 0.0, 1e-9);
    verify_orthogonal(&result, &u, 2, 1e-9);
}

/// u = [3, 0], v = [4, 5] → factor = 12/9 = 4/3, result = [0, 5].
#[test]
fn test_gs_2d() {
    let u = [3.0, 0.0];
    let v = [4.0, 5.0];
    let result = gram_schmidt_step(&u, &v, 2);
    assert_approx_eq(result[0], 0.0, 1e-9);
    assert_approx_eq(result[1], 5.0, 1e-9);
    verify_orthogonal(&result, &u, 2, 1e-9);
    verify_gs_step(&result, &u, &v, 2, 1e-9);
}

/// u = [1, 2, 3], v = [4, 5, 6].
/// dot_uu = 1+4+9 = 14, dot_vu = 4+10+18 = 32, factor = 32/14 = 16/7.
#[test]
fn test_gs_3d() {
    let u = [1.0, 2.0, 3.0];
    let v = [4.0, 5.0, 6.0];
    let result = gram_schmidt_step(&u, &v, 3);
    verify_orthogonal(&result, &u, 3, 1e-9);
    verify_gs_step(&result, &u, &v, 3, 1e-9);
}

// ============================================================================
// Higher-dimensional tests
// ============================================================================

/// u = [1, 0, 0], v = [2, 3, 4] → factor = 2, result = [0, 3, 4].
#[test]
fn test_gs_unit_u() {
    let u = [1.0, 0.0, 0.0];
    let v = [2.0, 3.0, 4.0];
    let result = gram_schmidt_step(&u, &v, 3);
    assert_approx_eq(result[0], 0.0, 1e-9);
    assert_approx_eq(result[1], 3.0, 1e-9);
    assert_approx_eq(result[2], 4.0, 1e-9);
    verify_orthogonal(&result, &u, 3, 1e-9);
    verify_gs_step(&result, &u, &v, 3, 1e-9);
}

/// Orthogonality check for 4D vectors.
#[test]
fn test_gs_4d_orthogonality() {
    let u = [1.0, 2.0, 3.0, 4.0];
    let v = [5.0, 6.0, 7.0, 8.0];
    let result = gram_schmidt_step(&u, &v, 4);
    verify_orthogonal(&result, &u, 4, 1e-9);
    verify_gs_step(&result, &u, &v, 4, 1e-9);
}

/// Verify that the result satisfies v - result = factor * u (i.e. parallel to u).
#[test]
fn test_gs_difference_parallel_to_u() {
    let u = [2.0, 1.0, 3.0];
    let v = [1.0, 4.0, 5.0];
    let result = gram_schmidt_step(&u, &v, 3);
    // v - result should be a scalar multiple of u
    let mut diff: Vec<f64> = Vec::new();
    let mut i: usize = 0;
    while i < 3 {
        diff.push(v[i] - result[i]);
        i += 1;
    }
    // diff should be parallel to u: diff[i] / u[i] should be constant
    let factor = diff[0] / u[0];
    let mut i: usize = 1;
    while i < 3 {
        assert_approx_eq(diff[i] / u[i], factor, 1e-9);
        i += 1;
    }
    verify_orthogonal(&result, &u, 3, 1e-9);
}
