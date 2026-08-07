use matrix_ldlt::ldlt_decomposition;

/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// Multiply two n×n matrices (row-major flat slices).
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

/// Compute the transpose of an n×n matrix.
fn matrix_transpose(a: &[f64], n: usize) -> Vec<f64> {
    let mut t: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < n * n {
        t.push(0.0);
        idx += 1;
    }
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            t[j * n + i] = a[i * n + j];
            j += 1;
        }
        i += 1;
    }
    t
}

/// Extract L (unit lower triangular) from the combined LDLᵀ matrix.
/// L[i][i] = 1, L[i][j] = ld[i*n+j] for i > j, L[i][j] = 0 for i < j.
fn extract_l(ld: &[f64], n: usize) -> Vec<f64> {
    let mut l: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < n * n {
        l.push(0.0);
        idx += 1;
    }
    let mut i: usize = 0;
    while i < n {
        l[i * n + i] = 1.0; // unit diagonal
        let mut j: usize = 0;
        while j < i {
            l[i * n + j] = ld[i * n + j];
            j += 1;
        }
        i += 1;
    }
    l
}

/// Extract D (diagonal matrix, stored as full n×n with zeros off-diagonal).
fn extract_d(ld: &[f64], n: usize) -> Vec<f64> {
    let mut d: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < n * n {
        d.push(0.0);
        idx += 1;
    }
    let mut i: usize = 0;
    while i < n {
        d[i * n + i] = ld[i * n + i];
        i += 1;
    }
    d
}

/// Verify that A = L * D * Lᵀ (within tolerance).
fn verify_ldlt(a: &[f64], ld: &[f64], n: usize, tol: f64) {
    let l = extract_l(ld, n);
    let d = extract_d(ld, n);
    let lt = matrix_transpose(&l, n); // Lᵀ
    let dl = matrix_mul(&d, &lt, n); // D * Lᵀ
    let ldl = matrix_mul(&l, &dl, n); // L * D * Lᵀ
    let mut i: usize = 0;
    while i < n * n {
        assert_approx_eq(ldl[i], a[i], tol);
        i += 1;
    }
}

// ============================================================================
// Basic tests
// ============================================================================

/// n = 0: empty matrix.
#[test]
fn test_ldlt_empty() {
    let result = ldlt_decomposition(&[], 0);
    assert!(result.ld.len() == 0);
}

/// n = 1: [[4]] → D = [4], L = [[1]].
#[test]
fn test_ldlt_1x1() {
    let a = [4.0];
    let result = ldlt_decomposition(&a, 1);
    assert!(result.ld.len() == 1);
    assert_approx_eq(result.ld[0], 4.0, 1e-9); // D[0] = 4
    verify_ldlt(&a, &result.ld, 1, 1e-9);
}

/// n = 1: [[0]] → panic (singular).
#[test]
#[should_panic(expected = "matrix is singular")]
fn test_ldlt_1x1_singular() {
    let a = [0.0];
    let _ = ldlt_decomposition(&a, 1);
}

/// n = 2: [[4, 1], [1, 3]] — SPD matrix.
#[test]
fn test_ldlt_2x2_spd() {
    let a = [4.0, 1.0, 1.0, 3.0];
    let result = ldlt_decomposition(&a, 2);
    assert!(result.ld.len() == 4);
    // D[0] = 4
    assert_approx_eq(result.ld[0], 4.0, 1e-9);
    // L[0][1] = 0 (above diagonal)
    assert_approx_eq(result.ld[1], 0.0, 1e-9);
    // L[1][0] = 1/4 = 0.25
    assert_approx_eq(result.ld[2], 0.25, 1e-9);
    // D[1] = 3 - 0.25*4*0.25 = 2.75
    assert_approx_eq(result.ld[3], 2.75, 1e-9);
    verify_ldlt(&a, &result.ld, 2, 1e-9);
}

/// n = 2: identity → L = I, D = I.
#[test]
fn test_ldlt_2x2_identity() {
    let a = [1.0, 0.0, 0.0, 1.0];
    let result = ldlt_decomposition(&a, 2);
    assert_approx_eq(result.ld[0], 1.0, 1e-9); // D[0]
    assert_approx_eq(result.ld[2], 0.0, 1e-9); // L[1][0]
    assert_approx_eq(result.ld[3], 1.0, 1e-9); // D[1]
    verify_ldlt(&a, &result.ld, 2, 1e-9);
}

/// n = 2: [[1, 2], [2, 1]] — indefinite (det = -3), LDLᵀ handles this.
#[test]
fn test_ldlt_2x2_indefinite() {
    let a = [1.0, 2.0, 2.0, 1.0];
    let result = ldlt_decomposition(&a, 2);
    // D[0] = 1
    assert_approx_eq(result.ld[0], 1.0, 1e-9);
    // L[1][0] = 2/1 = 2
    assert_approx_eq(result.ld[2], 2.0, 1e-9);
    // D[1] = 1 - 2*1*2 = -3
    assert_approx_eq(result.ld[3], -3.0, 1e-9);
    verify_ldlt(&a, &result.ld, 2, 1e-9);
}

/// n = 2: [[0, 1], [1, 0]] → zero pivot at D[0] (panic).
#[test]
#[should_panic(expected = "matrix is singular")]
fn test_ldlt_2x2_singular() {
    let a = [0.0, 1.0, 1.0, 0.0];
    let _ = ldlt_decomposition(&a, 2);
}

// ============================================================================
// 3×3 tests
// ============================================================================

/// n = 3: identity.
#[test]
fn test_ldlt_3x3_identity() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    ];
    let result = ldlt_decomposition(&a, 3);
    verify_ldlt(&a, &result.ld, 3, 1e-9);
}

/// n = 3: SPD matrix [[4,2,1],[2,3,1],[1,1,2]].
#[test]
fn test_ldlt_3x3_spd() {
    #[rustfmt::skip]
    let a = [
        4.0, 2.0, 1.0,
        2.0, 3.0, 1.0,
        1.0, 1.0, 2.0,
    ];
    let result = ldlt_decomposition(&a, 3);
    verify_ldlt(&a, &result.ld, 3, 1e-9);
}

/// n = 3: diagonal matrix [[2,0,0],[0,3,0],[0,0,5]].
#[test]
fn test_ldlt_3x3_diagonal() {
    #[rustfmt::skip]
    let a = [
        2.0, 0.0, 0.0,
        0.0, 3.0, 0.0,
        0.0, 0.0, 5.0,
    ];
    let result = ldlt_decomposition(&a, 3);
    assert_approx_eq(result.ld[0], 2.0, 1e-9); // D[0]
    assert_approx_eq(result.ld[4], 3.0, 1e-9); // D[1]
    assert_approx_eq(result.ld[8], 5.0, 1e-9); // D[2]
    verify_ldlt(&a, &result.ld, 3, 1e-9);
}

/// n = 3: indefinite [[1,2,0],[2,1,0],[0,0,4]] (D has mixed signs).
#[test]
fn test_ldlt_3x3_indefinite() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 0.0,
        2.0, 1.0, 0.0,
        0.0, 0.0, 4.0,
    ];
    let result = ldlt_decomposition(&a, 3);
    // D[0] = 1 > 0, D[1] = 1 - 4 = -3 < 0, D[2] = 4 > 0
    assert_approx_eq(result.ld[0], 1.0, 1e-9);
    assert_approx_eq(result.ld[4], -3.0, 1e-9);
    assert_approx_eq(result.ld[8], 4.0, 1e-9);
    verify_ldlt(&a, &result.ld, 3, 1e-9);
}

/// n = 3: [[1,2,3],[2,4,5],[3,5,6]] — singular (third pivot zero).
#[test]
#[should_panic(expected = "matrix is singular")]
fn test_ldlt_3x3_singular() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        2.0, 4.0, 5.0,
        3.0, 5.0, 6.0,
    ];
    let _ = ldlt_decomposition(&a, 3);
}

/// n = 3: general SPD matrix [[25,15,-5],[15,18,0],[-5,0,11]].
#[test]
fn test_ldlt_3x3_spd2() {
    #[rustfmt::skip]
    let a = [
        25.0, 15.0, -5.0,
        15.0, 18.0,  0.0,
        -5.0,  0.0, 11.0,
    ];
    let result = ldlt_decomposition(&a, 3);
    verify_ldlt(&a, &result.ld, 3, 1e-9);
}

// ============================================================================
// 4×4 tests
// ============================================================================

/// n = 4: SPD matrix.
#[test]
fn test_ldlt_4x4_spd() {
    #[rustfmt::skip]
    let a = [
        10.0,  1.0,  2.0,  3.0,
         1.0,  9.0,  1.0,  2.0,
         2.0,  1.0,  8.0,  1.0,
         3.0,  2.0,  1.0,  7.0,
    ];
    let result = ldlt_decomposition(&a, 4);
    verify_ldlt(&a, &result.ld, 4, 1e-9);
}

/// n = 4: diagonal matrix.
#[test]
fn test_ldlt_4x4_diagonal() {
    #[rustfmt::skip]
    let a = [
        2.0, 0.0, 0.0, 0.0,
        0.0, 3.0, 0.0, 0.0,
        0.0, 0.0, 5.0, 0.0,
        0.0, 0.0, 0.0, 7.0,
    ];
    let result = ldlt_decomposition(&a, 4);
    verify_ldlt(&a, &result.ld, 4, 1e-9);
}

/// n = 4: indefinite (D has mixed signs).
#[test]
fn test_ldlt_4x4_indefinite() {
    #[rustfmt::skip]
    let a = [
         1.0,  2.0,  0.0,  0.0,
         2.0,  1.0,  0.0,  0.0,
         0.0,  0.0,  4.0,  1.0,
         0.0,  0.0,  1.0,  3.0,
    ];
    let result = ldlt_decomposition(&a, 4);
    verify_ldlt(&a, &result.ld, 4, 1e-9);
}

// ============================================================================
// Structural tests
// ============================================================================

/// Verify L is unit lower triangular and D entries are on diagonal.
#[test]
fn test_ldlt_structure() {
    #[rustfmt::skip]
    let a = [
        4.0, 2.0, 1.0,
        2.0, 3.0, 1.0,
        1.0, 1.0, 2.0,
    ];
    let result = ldlt_decomposition(&a, 3);
    let ld = &result.ld;

    // Check zeros above diagonal
    let mut i: usize = 0;
    while i < 3 {
        let mut j: usize = i + 1;
        while j < 3 {
            assert_approx_eq(ld[i * 3 + j], 0.0, 1e-9);
            j += 1;
        }
        i += 1;
    }

    // Verify extract_l has 1s on diagonal
    let l = extract_l(ld, 3);
    let mut i: usize = 0;
    while i < 3 {
        assert_approx_eq(l[i * 3 + i], 1.0, 1e-9);
        i += 1;
    }

    // Verify D is stored on diagonal (non-zero per assert)
    let mut i: usize = 0;
    while i < 3 {
        assert!(ld[i * 3 + i] != 0.0);
        i += 1;
    }
}
