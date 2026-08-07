use matrix_lu::lu_decomposition;

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

/// Extract L (unit lower triangular) from the combined LU matrix.
fn extract_l(lu: &[f64], n: usize) -> Vec<f64> {
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
            l[i * n + j] = lu[i * n + j];
            j += 1;
        }
        i += 1;
    }
    l
}

/// Extract U (upper triangular) from the combined LU matrix.
fn extract_u(lu: &[f64], n: usize) -> Vec<f64> {
    let mut u: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < n * n {
        u.push(0.0);
        idx += 1;
    }
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = i;
        while j < n {
            u[i * n + j] = lu[i * n + j];
            j += 1;
        }
        i += 1;
    }
    u
}

/// Apply the permutation `perm` to the rows of `a` (P * A).
fn permute_rows(a: &[f64], perm: &[usize], n: usize) -> Vec<f64> {
    let mut pa: Vec<f64> = Vec::new();
    let mut idx: usize = 0;
    while idx < n * n {
        pa.push(0.0);
        idx += 1;
    }
    let mut i: usize = 0;
    while i < n {
        let src = perm[i];
        let mut j: usize = 0;
        while j < n {
            pa[i * n + j] = a[src * n + j];
            j += 1;
        }
        i += 1;
    }
    pa
}

/// Verify that P * A = L * U (within tolerance).
fn verify_lu(a: &[f64], lu: &[f64], perm: &[usize], n: usize, tol: f64) {
    let l = extract_l(lu, n);
    let u = extract_u(lu, n);
    let lu_prod = matrix_mul(&l, &u, n); // L * U
    let pa = permute_rows(a, perm, n); // P * A
    let mut i: usize = 0;
    while i < n * n {
        assert_approx_eq(lu_prod[i], pa[i], tol);
        i += 1;
    }
}

// ============================================================================
// Basic tests
// ============================================================================

/// n = 0: empty matrix.
#[test]
fn test_lu_empty() {
    let result = lu_decomposition(&[], 0);
    assert!(result.lu.len() == 0);
    assert!(result.perm.len() == 0);
}

/// n = 1: [[5]] → L = [[1]], U = [[5]], perm = [0].
#[test]
fn test_lu_1x1() {
    let a = [5.0];
    let result = lu_decomposition(&a, 1);
    assert!(result.lu.len() == 1);
    assert!(result.perm.len() == 1);
    assert!(result.perm[0] == 0);
    assert_approx_eq(result.lu[0], 5.0, 1e-9);
    verify_lu(&a, &result.lu, &result.perm, 1, 1e-9);
}

/// n = 1: singular [[0]] → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_lu_1x1_singular() {
    let a = [0.0];
    let _ = lu_decomposition(&a, 1);
}

/// n = 2: [[3, 1], [1, 2]] — no pivoting needed.
#[test]
fn test_lu_2x2_no_pivot() {
    let a = [3.0, 1.0, 1.0, 2.0];
    let result = lu_decomposition(&a, 2);
    assert!(result.lu.len() == 4);
    assert!(result.perm.len() == 2);
    // No pivoting expected: |3| > |1|
    assert!(result.perm[0] == 0);
    assert!(result.perm[1] == 1);
    // U[0,0] = 3, U[0,1] = 1
    assert_approx_eq(result.lu[0], 3.0, 1e-9);
    assert_approx_eq(result.lu[1], 1.0, 1e-9);
    // L[1,0] = 1/3, U[1,1] = 2 - 1/3 * 1 = 5/3
    assert_approx_eq(result.lu[2], 1.0 / 3.0, 1e-9);
    assert_approx_eq(result.lu[3], 2.0 - (1.0 / 3.0) * 1.0, 1e-9);
    verify_lu(&a, &result.lu, &result.perm, 2, 1e-9);
}

/// n = 2: [[2, 1], [4, 3]] — pivoting required.
/// |4| > |2| in column 0, so rows are swapped.
#[test]
fn test_lu_2x2_with_pivot() {
    let a = [2.0, 1.0, 4.0, 3.0];
    let result = lu_decomposition(&a, 2);
    assert!(result.lu.len() == 4);
    assert!(result.perm.len() == 2);
    // Pivoting: row 0 swapped with row 1
    assert!(result.perm[0] == 1);
    assert!(result.perm[1] == 0);
    // After swap, pivot = 4
    assert_approx_eq(result.lu[0], 4.0, 1e-9);
    assert_approx_eq(result.lu[1], 3.0, 1e-9);
    // L[1,0] = 2.0 / 4.0 = 0.5
    assert_approx_eq(result.lu[2], 0.5, 1e-9);
    // U[1,1] = 1.0 - 0.5 * 3.0 = -0.5
    assert_approx_eq(result.lu[3], -0.5, 1e-9);
    verify_lu(&a, &result.lu, &result.perm, 2, 1e-9);
}

/// n = 2: identity → L = I, U = I, perm = [0, 1].
#[test]
fn test_lu_2x2_identity() {
    let a = [1.0, 0.0, 0.0, 1.0];
    let result = lu_decomposition(&a, 2);
    assert!(result.perm[0] == 0);
    assert!(result.perm[1] == 1);
    verify_lu(&a, &result.lu, &result.perm, 2, 1e-9);
}

/// n = 2: singular [[1, 2], [2, 4]] → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_lu_2x2_singular() {
    let a = [1.0, 2.0, 2.0, 4.0];
    let _ = lu_decomposition(&a, 2);
}

// ============================================================================
// 3×3 tests
// ============================================================================

/// n = 3: identity.
#[test]
fn test_lu_3x3_identity() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    ];
    let result = lu_decomposition(&a, 3);
    assert!(result.perm[0] == 0);
    assert!(result.perm[1] == 1);
    assert!(result.perm[2] == 2);
    verify_lu(&a, &result.lu, &result.perm, 3, 1e-9);
}

/// n = 3: SPD matrix [[4,2,1],[2,3,1],[1,1,2]] — no pivoting needed.
#[test]
fn test_lu_3x3_spd() {
    #[rustfmt::skip]
    let a = [
        4.0, 2.0, 1.0,
        2.0, 3.0, 1.0,
        1.0, 1.0, 2.0,
    ];
    let result = lu_decomposition(&a, 3);
    // No pivoting expected for this diagonally dominant matrix.
    assert!(result.perm[0] == 0);
    assert!(result.perm[1] == 1);
    assert!(result.perm[2] == 2);
    verify_lu(&a, &result.lu, &result.perm, 3, 1e-9);
}

/// n = 3: matrix requiring pivoting (zeros in first column).
/// [[0,1,2],[0,3,4],[5,6,7]]
#[test]
fn test_lu_3x3_zeros_first_col() {
    #[rustfmt::skip]
    let a = [
        0.0, 1.0, 2.0,
        0.0, 3.0, 4.0,
        5.0, 6.0, 7.0,
    ];
    let result = lu_decomposition(&a, 3);
    // Must have pivoted row 2 to the top.
    assert!(result.perm[0] == 2);
    verify_lu(&a, &result.lu, &result.perm, 3, 1e-9);
}

/// n = 3: general matrix [[1,2,3],[0,1,4],[5,6,0]].
#[test]
fn test_lu_3x3_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        0.0, 1.0, 4.0,
        5.0, 6.0, 0.0,
    ];
    let result = lu_decomposition(&a, 3);
    verify_lu(&a, &result.lu, &result.perm, 3, 1e-9);
}

/// n = 3: singular (row of zeros) → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_lu_3x3_singular() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        0.0, 0.0, 0.0,
    ];
    let _ = lu_decomposition(&a, 3);
}

/// n = 3: singular (linearly dependent rows) → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_lu_3x3_singular2() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        2.0, 4.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let _ = lu_decomposition(&a, 3);
}

// ============================================================================
// 4×4 tests
// ============================================================================

/// n = 4: general invertible matrix.
#[test]
fn test_lu_4x4_general() {
    #[rustfmt::skip]
    let a = [
        1.0, 0.0, 2.0, -1.0,
        3.0, 0.0, 0.0,  5.0,
        2.0, 1.0, 4.0, -3.0,
        1.0, 0.0, 5.0,  0.0,
    ];
    let result = lu_decomposition(&a, 4);
    verify_lu(&a, &result.lu, &result.perm, 4, 1e-9);
}

/// n = 4: diagonal matrix [[2,0,0,0],[0,3,0,0],[0,0,5,0],[0,0,0,7]].
#[test]
fn test_lu_4x4_diagonal() {
    #[rustfmt::skip]
    let a = [
        2.0, 0.0, 0.0, 0.0,
        0.0, 3.0, 0.0, 0.0,
        0.0, 0.0, 5.0, 0.0,
        0.0, 0.0, 0.0, 7.0,
    ];
    let result = lu_decomposition(&a, 4);
    assert!(result.perm[0] == 0);
    assert!(result.perm[1] == 1);
    assert!(result.perm[2] == 2);
    assert!(result.perm[3] == 3);
    verify_lu(&a, &result.lu, &result.perm, 4, 1e-9);
}

/// n = 4: singular (row of zeros) → panic.
#[test]
#[should_panic(expected = "singular matrix")]
fn test_lu_4x4_singular() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        0.0, 0.0, 0.0, 0.0,
        9.0, 10.0, 11.0, 12.0,
    ];
    let _ = lu_decomposition(&a, 4);
}

// ============================================================================
// Triangular structure checks
// ============================================================================

/// Verify L is unit lower triangular, U is upper triangular.
#[test]
fn test_lu_triangular_structure() {
    #[rustfmt::skip]
    let a = [
        4.0, 2.0, 1.0,
        2.0, 3.0, 1.0,
        1.0, 1.0, 2.0,
    ];
    let result = lu_decomposition(&a, 3);
    let l = extract_l(&result.lu, 3);
    let u = extract_u(&result.lu, 3);

    // L: check unit diagonal and zeros above diagonal
    let mut i: usize = 0;
    while i < 3 {
        assert_approx_eq(l[i * 3 + i], 1.0, 1e-9);
        let mut j: usize = i + 1;
        while j < 3 {
            assert_approx_eq(l[i * 3 + j], 0.0, 1e-9);
            j += 1;
        }
        i += 1;
    }

    // U: check zeros below diagonal
    let mut i: usize = 0;
    while i < 3 {
        let mut j: usize = 0;
        while j < i {
            assert_approx_eq(u[i * 3 + j], 0.0, 1e-9);
            j += 1;
        }
        i += 1;
    }
}
