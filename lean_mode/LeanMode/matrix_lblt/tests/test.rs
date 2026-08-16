use matrix_lblt::lblt_decomposition;

/// Multiply two n×n matrices (column-major).
fn mat_mul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut c = vec![0.0; n * n];
    let mut col: usize = 0;
    while col < n {
        let mut row: usize = 0;
        while row < n {
            let mut sum: f64 = 0.0;
            let mut k: usize = 0;
            while k < n {
                sum = sum + a[row + k * n] * b[k + col * n];
                k += 1;
            }
            c[row + col * n] = sum;
            row += 1;
        }
        col += 1;
    }
    c
}

/// Transpose an n×n matrix (column-major).
fn transpose(a: &[f64], n: usize) -> Vec<f64> {
    let mut t = vec![0.0; n * n];
    let mut col: usize = 0;
    while col < n {
        let mut row: usize = 0;
        while row < n {
            t[row + col * n] = a[col + row * n];
            row += 1;
        }
        col += 1;
    }
    t
}

/// Check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// Verify A = L · B · Lᵀ and that B is tridiagonal (bandwidth 1).
fn check_decomposition(a: &[f64], n: usize, tol: f64) {
    let r = lblt_decomposition(a, n);
    assert_eq!(r.l.len(), n * n);
    assert_eq!(r.d.len(), n * n);

    // A = L · B · Lᵀ.
    let lb = mat_mul(&r.l, &r.d, n);
    let lbt = mat_mul(&lb, &transpose(&r.l, n), n);
    let mut i: usize = 0;
    while i < n * n {
        assert_approx_eq(lbt[i], a[i], tol);
        i += 1;
    }

    // B tridiagonal: B[i][j] = 0 for i > j+1 and for i+1 < j.
    let mut col: usize = 0;
    while col < n {
        let mut row: usize = 0;
        while row < n {
            let below = row > col + 1;
            let above = row + 1 < col;
            if below || above {
                assert_approx_eq(r.d[row + col * n], 0.0, tol);
            }
            row += 1;
        }
        col += 1;
    }
}

#[test]
fn test_1x1() {
    let a = [5.0];
    check_decomposition(&a, 1, 1e-12);
}

#[test]
fn test_2x2_spd() {
    // [[4, 1], [1, 3]] (SPD).
    let a = [4.0, 1.0, 1.0, 3.0];
    check_decomposition(&a, 2, 1e-12);
}

#[test]
fn test_2x2_indefinite() {
    // [[1, 2], [2, 1]] — indefinite, forces a 2×2 pivot block.
    let a = [1.0, 2.0, 2.0, 1.0];
    check_decomposition(&a, 2, 1e-12);
}

#[test]
fn test_2x2_exchange() {
    // [[0, 1], [1, 0]] — zero diagonal, forces a 2×2 pivot block.
    let a = [0.0, 1.0, 1.0, 0.0];
    check_decomposition(&a, 2, 1e-12);
}

#[test]
fn test_3x3_spd() {
    #[rustfmt::skip]
    let a = [
        4.0, 1.0, 2.0,
        1.0, 3.0, 1.0,
        2.0, 1.0, 5.0,
    ];
    check_decomposition(&a, 3, 1e-10);
}

#[test]
fn test_3x3_indefinite() {
    // Indefinite: eigenvalues of mixed sign.
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0,
        2.0, -1.0, 1.0,
        3.0, 1.0, 2.0,
    ];
    check_decomposition(&a, 3, 1e-10);
}

#[test]
fn test_3x3_exchange() {
    // Anti-diagonal permutation matrix (zero diagonal everywhere).
    #[rustfmt::skip]
    let a = [
        0.0, 0.0, 1.0,
        0.0, 1.0, 0.0,
        1.0, 0.0, 0.0,
    ];
    check_decomposition(&a, 3, 1e-10);
}

#[test]
fn test_4x4_indefinite() {
    #[rustfmt::skip]
    let a = [
        1.0, 2.0, 3.0, 4.0,
        2.0, -1.0, 1.0, 2.0,
        3.0, 1.0, 2.0, -3.0,
        4.0, 2.0, -3.0, 1.0,
    ];
    check_decomposition(&a, 4, 1e-9);
}

#[test]
fn test_5x5_indefinite() {
    // Indefinite with a zero diagonal entry.
    #[rustfmt::skip]
    let a = [
        0.0, 1.0, 2.0, 3.0, 4.0,
        1.0, 2.0, 1.0, 2.0, 3.0,
        2.0, 1.0, 0.0, 1.0, 2.0,
        3.0, 2.0, 1.0, 3.0, 1.0,
        4.0, 3.0, 2.0, 1.0, 4.0,
    ];
    check_decomposition(&a, 5, 1e-9);
}

#[test]
fn test_identity() {
    let a = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    check_decomposition(&a, 3, 1e-12);
}

#[test]
fn test_zero() {
    let a = [0.0; 9];
    check_decomposition(&a, 3, 1e-12);
}

#[test]
fn test_empty() {
    let a: Vec<f64> = vec![];
    let r = lblt_decomposition(&a, 0);
    assert!(r.l.len() == 0);
    assert!(r.d.len() == 0);
}
