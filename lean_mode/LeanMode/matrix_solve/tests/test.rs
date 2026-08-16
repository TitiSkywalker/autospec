use matrix_solve::solve_lower_triangular;
use matrix_solve::solve_upper_triangular;

/// Matrix-vector product y = M · v (M is n×n column-major, v is length n).
fn mat_vec_mul(m: &[f64], v: &[f64], n: usize) -> Vec<f64> {
    let mut y = vec![0.0; n];
    let mut i: usize = 0;
    while i < n {
        let mut sum: f64 = 0.0;
        let mut k: usize = 0;
        while k < n {
            sum = sum + m[i + k * n] * v[k];
            k += 1;
        }
        y[i] = sum;
        i += 1;
    }
    y
}

/// Check two length-n vectors are equal element-by-element.
fn check_vec_eq(a: &[f64], b: &[f64], n: usize) {
    let mut i: usize = 0;
    while i < n {
        assert!(a[i] == b[i]);
        i += 1;
    }
}

// ============================================================================
// Lower triangular
// ============================================================================

#[test]
fn test_lower_1x1() {
    let l = [5.0];
    let b = [15.0];
    let x = solve_lower_triangular(&l, &b, 1);
    assert!(x[0] == 3.0);
}

#[test]
fn test_lower_2x2() {
    // L = [[1, 0], [2, 3]] (column-major).
    let l = [1.0, 2.0, 0.0, 3.0];
    let x_true = [4.0, 5.0];
    let b = mat_vec_mul(&l, &x_true, 2);
    let x = solve_lower_triangular(&l, &b, 2);
    check_vec_eq(&x, &x_true, 2);
}

#[test]
fn test_lower_3x3() {
    // L = [[1, 0, 0], [2, 3, 0], [4, 5, 6]] (column-major).
    #[rustfmt::skip]
    let l = [
        1.0, 2.0, 4.0,
        0.0, 3.0, 5.0,
        0.0, 0.0, 6.0,
    ];
    let x_true = [1.0, 2.0, 3.0];
    let b = mat_vec_mul(&l, &x_true, 3);
    let x = solve_lower_triangular(&l, &b, 3);
    check_vec_eq(&x, &x_true, 3);
}

#[test]
fn test_lower_4x4() {
    // L = [[1,0,0,0],[1,2,0,0],[1,2,3,0],[1,2,3,4]] (column-major).
    #[rustfmt::skip]
    let l = [
        1.0, 1.0, 1.0, 1.0,
        0.0, 2.0, 2.0, 2.0,
        0.0, 0.0, 3.0, 3.0,
        0.0, 0.0, 0.0, 4.0,
    ];
    let x_true = [5.0, -3.0, 7.0, 2.0];
    let b = mat_vec_mul(&l, &x_true, 4);
    let x = solve_lower_triangular(&l, &b, 4);
    check_vec_eq(&x, &x_true, 4);
}

// ============================================================================
// Upper triangular
// ============================================================================

#[test]
fn test_upper_1x1() {
    let u = [5.0];
    let b = [15.0];
    let x = solve_upper_triangular(&u, &b, 1);
    assert!(x[0] == 3.0);
}

#[test]
fn test_upper_2x2() {
    // U = [[1, 2], [0, 3]] (column-major).
    let u = [1.0, 0.0, 2.0, 3.0];
    let x_true = [4.0, 5.0];
    let b = mat_vec_mul(&u, &x_true, 2);
    let x = solve_upper_triangular(&u, &b, 2);
    check_vec_eq(&x, &x_true, 2);
}

#[test]
fn test_upper_3x3() {
    // U = [[1, 2, 3], [0, 4, 5], [0, 0, 6]] (column-major).
    #[rustfmt::skip]
    let u = [
        1.0, 0.0, 0.0,
        2.0, 4.0, 0.0,
        3.0, 5.0, 6.0,
    ];
    let x_true = [1.0, 2.0, 3.0];
    let b = mat_vec_mul(&u, &x_true, 3);
    let x = solve_upper_triangular(&u, &b, 3);
    check_vec_eq(&x, &x_true, 3);
}

#[test]
fn test_upper_4x4() {
    // U = [[1,2,3,4],[0,2,3,4],[0,0,3,4],[0,0,0,4]] (column-major).
    #[rustfmt::skip]
    let u = [
        1.0, 0.0, 0.0, 0.0,
        2.0, 2.0, 0.0, 0.0,
        3.0, 3.0, 3.0, 0.0,
        4.0, 4.0, 4.0, 4.0,
    ];
    let x_true = [5.0, -3.0, 7.0, 2.0];
    let b = mat_vec_mul(&u, &x_true, 4);
    let x = solve_upper_triangular(&u, &b, 4);
    check_vec_eq(&x, &x_true, 4);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn test_empty() {
    let x = solve_lower_triangular(&[], &[], 0);
    assert!(x.is_empty());
    let y = solve_upper_triangular(&[], &[], 0);
    assert!(y.is_empty());
}

#[test]
#[should_panic]
fn test_lower_singular_panics() {
    // L = [[0, 0], [1, 2]] has a zero diagonal.
    let l = [0.0, 1.0, 0.0, 2.0];
    let b = [1.0, 2.0];
    let _ = solve_lower_triangular(&l, &b, 2);
}
