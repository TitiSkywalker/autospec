use matrix_cholesky::cholesky_decomposition;

/// Element at (row, col) of a column-major n×n matrix.
fn at(m: &[f64], n: usize, row: usize, col: usize) -> f64 {
    m[row + col * n]
}

/// Reconstruct A = L · Lᵀ (column-major).
fn reconstruct(l: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                // A[i][j] = Σ_k L[i][k] · L[j][k]
                sum += at(l, n, i, k) * at(l, n, j, k);
            }
            out[i + j * n] = sum;
        }
    }
    out
}

/// Verify L is lower-triangular: L[i][j] = 0 for i < j.
fn lower_triangular(l: &[f64], n: usize, tol: f64) -> bool {
    for j in 0..n {
        for i in 0..j {
            if at(l, n, i, j).abs() > tol {
                return false;
            }
        }
    }
    true
}

/// Verify the diagonal of L is strictly positive.
fn positive_diagonal(l: &[f64], n: usize) -> bool {
    for i in 0..n {
        if at(l, n, i, i) <= 0.0 {
            return false;
        }
    }
    true
}

fn close(a: &[f64], b: &[f64], tol: f64) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if (a[i] - b[i]).abs() > tol {
            return false;
        }
    }
    true
}

fn check_decomposition(a: &[f64], n: usize, tol: f64) {
    let r = cholesky_decomposition(a, n);
    assert_eq!(r.l.len(), n * n);
    assert!(lower_triangular(&r.l, n, tol));
    assert!(positive_diagonal(&r.l, n));
    assert!(close(&reconstruct(&r.l, n), a, tol));
}

#[test]
fn test_1x1() {
    let m = vec![5.0];
    let r = cholesky_decomposition(&m, 1);
    assert_eq!(r.l.len(), 1);
    assert!((r.l[0] - 5.0_f64.sqrt()).abs() < 1e-12);
    check_decomposition(&m, 1, 1e-12);
}

#[test]
fn test_2x2() {
    // [[4, 1], [1, 3]] — SPD.
    let m = vec![4.0, 1.0, 1.0, 3.0];
    check_decomposition(&m, 2, 1e-12);
}

#[test]
fn test_3x3_classic() {
    // Classic example: A = [[4, 12, -16], [12, 37, -43], [-16, -43, 98]]
    // with L = [[2, 0, 0], [6, 1, 0], [-8, 5, 3]].
    let m = vec![
        4.0, 12.0, -16.0,
        12.0, 37.0, -43.0,
        -16.0, -43.0, 98.0,
    ];
    let r = cholesky_decomposition(&m, 3);
    check_decomposition(&m, 3, 1e-12);
    // Spot-check the known factor.
    assert!((at(&r.l, 3, 0, 0) - 2.0).abs() < 1e-12);
    assert!((at(&r.l, 3, 1, 0) - 6.0).abs() < 1e-12);
    assert!((at(&r.l, 3, 2, 0) - -8.0).abs() < 1e-12);
    assert!((at(&r.l, 3, 2, 2) - 3.0).abs() < 1e-12);
}

#[test]
fn test_3x3_diagonal() {
    // diag(1, 4, 9) → L = diag(1, 2, 3).
    let m = vec![
        1.0, 0.0, 0.0,
        0.0, 4.0, 0.0,
        0.0, 0.0, 9.0,
    ];
    let r = cholesky_decomposition(&m, 3);
    check_decomposition(&m, 3, 1e-12);
    assert!((at(&r.l, 3, 0, 0) - 1.0).abs() < 1e-12);
    assert!((at(&r.l, 3, 1, 1) - 2.0).abs() < 1e-12);
    assert!((at(&r.l, 3, 2, 2) - 3.0).abs() < 1e-12);
}

#[test]
fn test_4x4() {
    // SPD: A = B · Bᵀ where B is a lower-triangular-ish matrix, so Cholesky
    // factors it cleanly.
    #[rustfmt::skip]
    let m = vec![
        5.0, 1.0, 2.0, 3.0,
        1.0, 6.0, 4.0, 5.0,
        2.0, 4.0, 7.0, 6.0,
        3.0, 5.0, 6.0, 10.0,
    ];
    check_decomposition(&m, 4, 1e-10);
}

#[test]
fn test_5x5() {
    // Diagonally dominant SPD matrix.
    #[rustfmt::skip]
    let m = vec![
        10.0, 1.0, 2.0, 3.0, 4.0,
        1.0, 11.0, 2.0, 3.0, 4.0,
        2.0, 2.0, 12.0, 3.0, 4.0,
        3.0, 3.0, 3.0, 13.0, 4.0,
        4.0, 4.0, 4.0, 4.0, 14.0,
    ];
    check_decomposition(&m, 5, 1e-10);
}

#[test]
#[should_panic]
fn test_not_spd_panics() {
    // [[1, 2], [2, 1]] is indefinite (eigenvalues -1, 3), so Cholesky fails.
    let m = vec![1.0, 2.0, 2.0, 1.0];
    let _ = cholesky_decomposition(&m, 2);
}

#[test]
fn test_empty() {
    let m: Vec<f64> = vec![];
    let r = cholesky_decomposition(&m, 0);
    assert!(r.l.is_empty());
}
