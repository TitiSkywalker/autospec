/// Helper — compare two n×n matrices (row-major) element-wise.
fn assert_mat_eq(actual: &[f64], expected: &[f64], n: usize, tol: f64) {
    let mut i = 0;
    while i < n * n {
        let diff = (actual[i] - expected[i]).abs();
        assert!(diff <= tol);
        i += 1;
    }
}

/// n = 0: empty matrix, success, empty Jordan form.
#[test]
fn test_jordan_empty() {
    let r = matrix_jordan::jordan_form(&[], 0);
    assert!(r.success);
    assert_eq!(r.j.len(), 0);
}

/// n = 1: [[5]] → single 1×1 block [[5]].
#[test]
fn test_jordan_1x1() {
    let r = matrix_jordan::jordan_form(&[5.0], 1);
    assert!(r.success);
    assert_mat_eq(&r.j, &[5.0], 1, 0.001);
}

/// n = 2: [[2, 1], [0, 2]] → one 2×2 block (already in Jordan form).
#[test]
fn test_jordan_2x2_single_block() {
    let m = vec![2.0, 1.0, 0.0, 2.0];
    let r = matrix_jordan::jordan_form(&m, 2);
    assert!(r.success);
    assert_mat_eq(&r.j, &m, 2, 0.001);
}

/// n = 2: [[2, 0], [0, 2]] → two 1×1 blocks.
#[test]
fn test_jordan_2x2_diagonal() {
    let m = vec![2.0, 0.0, 0.0, 2.0];
    let r = matrix_jordan::jordan_form(&m, 2);
    assert!(r.success);
    assert_mat_eq(&r.j, &m, 2, 0.001);
}

/// n = 2: [[1, 2], [3, 4]] → eigenvalues (5 ± √33)/2 are irrational → failure.
#[test]
fn test_jordan_2x2_irrational() {
    let m = vec![1.0, 2.0, 3.0, 4.0];
    let r = matrix_jordan::jordan_form(&m, 2);
    assert!(!r.success);
    assert_eq!(r.j.len(), 4);
}

/// n = 3: [[2, 1, 0], [0, 2, 1], [0, 0, 2]] → one 3×3 block.
#[test]
fn test_jordan_3x3_single_block() {
    #[rustfmt::skip]
    let m = vec![
        2.0, 1.0, 0.0,
        0.0, 2.0, 1.0,
        0.0, 0.0, 2.0,
    ];
    let r = matrix_jordan::jordan_form(&m, 3);
    assert!(r.success);
    assert_mat_eq(&r.j, &m, 3, 0.001);
}

/// n = 3: diagonal 2, 3, 4 → three 1×1 blocks.
#[test]
fn test_jordan_3x3_diagonal() {
    #[rustfmt::skip]
    let m = vec![
        2.0, 0.0, 0.0,
        0.0, 3.0, 0.0,
        0.0, 0.0, 4.0,
    ];
    let r = matrix_jordan::jordan_form(&m, 3);
    assert!(r.success);
    assert_mat_eq(&r.j, &m, 3, 0.001);
}

/// n = 3: [[2, 1, 0], [0, 2, 0], [0, 0, 3]] → J₂(2) ⊕ J₁(3).
#[test]
fn test_jordan_3x3_mixed() {
    #[rustfmt::skip]
    let m = vec![
        2.0, 1.0, 0.0,
        0.0, 2.0, 0.0,
        0.0, 0.0, 3.0,
    ];
    let r = matrix_jordan::jordan_form(&m, 3);
    assert!(r.success);
    assert_mat_eq(&r.j, &m, 3, 0.001);
}

/// n = 3: [[0, -1, 0], [1, 0, 0], [0, 0, 5]] → char poly (x²+1)(x−5): the
/// eigenvalue 5 is rational but ±i are not → failure.
#[test]
fn test_jordan_3x3_partial_irrational() {
    #[rustfmt::skip]
    let m = vec![
        0.0, -1.0, 0.0,
        1.0, 0.0, 0.0,
        0.0, 0.0, 5.0,
    ];
    let r = matrix_jordan::jordan_form(&m, 3);
    assert!(!r.success);
    assert_eq!(r.j.len(), 9);
}
