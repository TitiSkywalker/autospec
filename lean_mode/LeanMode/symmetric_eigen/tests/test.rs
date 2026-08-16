use symmetric_eigen::*;

/// Element at (row, col) of a column-major n×n matrix.
fn at(m: &[f64], n: usize, row: usize, col: usize) -> f64 {
    m[row + col * n]
}

/// Reconstruct A = V · Λ · Vᵀ (all n×n, column-major).
fn reconstruct(v: &[f64], values: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += at(v, n, i, k) * values[k] * at(v, n, j, k);
            }
            out[i + j * n] = sum;
        }
    }
    out
}

/// Check Vᵀ V = I (columns of V are orthonormal).
fn orthogonal(v: &[f64], n: usize, tol: f64) -> bool {
    for i in 0..n {
        for j in 0..n {
            let mut dot = 0.0;
            for k in 0..n {
                dot += at(v, n, k, i) * at(v, n, k, j);
            }
            let expect = if i == j { 1.0 } else { 0.0 };
            if (dot - expect).abs() > tol {
                return false;
            }
        }
    }
    true
}

/// Check A · v_j ≈ λ_j · v_j for every eigenpair (the defining property).
fn eigen_property(a: &[f64], v: &[f64], values: &[f64], n: usize, tol: f64) -> bool {
    for j in 0..n {
        let lambda = values[j];
        for i in 0..n {
            let mut av = 0.0;
            for k in 0..n {
                av += at(a, n, i, k) * at(v, n, k, j);
            }
            if (av - lambda * at(v, n, i, j)).abs() > tol {
                return false;
            }
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

fn sorted(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

#[test]
fn test_1x1() {
    let m = vec![7.0];
    let r = symmetric_eigen(&m, 1, 1e-12, 100);
    assert_eq!(r.eigenvalues.len(), 1);
    assert_eq!(r.eigenvectors.len(), 1);
    assert!((r.eigenvalues[0] - 7.0).abs() < 1e-12);
    assert!((r.eigenvectors[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_2x2_diagonal() {
    let m = vec![2.0, 0.0, 0.0, 3.0];
    let r = symmetric_eigen(&m, 2, 1e-12, 100);
    let sv = sorted(r.eigenvalues.clone());
    assert!((sv[0] - 2.0).abs() < 1e-12, "λ₀ = {}", sv[0]);
    assert!((sv[1] - 3.0).abs() < 1e-12, "λ₁ = {}", sv[1]);
    assert!(orthogonal(&r.eigenvectors, 2, 1e-12));
}

#[test]
fn test_2x2_known_41() {
    // [[4, 1], [1, 4]] → eigenvalues 3 and 5.
    let m = vec![4.0, 1.0, 1.0, 4.0];
    let r = symmetric_eigen(&m, 2, 1e-12, 100);
    let sv = sorted(r.eigenvalues.clone());
    assert!((sv[0] - 3.0).abs() < 1e-10, "λ₀ = {}", sv[0]);
    assert!((sv[1] - 5.0).abs() < 1e-10, "λ₁ = {}", sv[1]);
    assert!(orthogonal(&r.eigenvectors, 2, 1e-10));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 2), &m, 1e-10));
    assert!(eigen_property(&m, &r.eigenvectors, &r.eigenvalues, 2, 1e-10));
}

#[test]
fn test_2x2_known_12() {
    // [[1, 2], [2, 1]] → eigenvalues -1 and 3.
    let m = vec![1.0, 2.0, 2.0, 1.0];
    let r = symmetric_eigen(&m, 2, 1e-12, 100);
    let sv = sorted(r.eigenvalues.clone());
    assert!((sv[0] - -1.0).abs() < 1e-10, "λ₀ = {}", sv[0]);
    assert!((sv[1] - 3.0).abs() < 1e-10, "λ₁ = {}", sv[1]);
    assert!(orthogonal(&r.eigenvectors, 2, 1e-10));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 2), &m, 1e-10));
    assert!(eigen_property(&m, &r.eigenvectors, &r.eigenvalues, 2, 1e-10));
}

#[test]
fn test_2x2_known_21() {
    // [[2, 1], [1, 3]] → eigenvalues (5 ± √5)/2 ≈ 3.618034, 1.381966.
    let m = vec![2.0, 1.0, 1.0, 3.0];
    let r = symmetric_eigen(&m, 2, 1e-12, 100);
    let sv = sorted(r.eigenvalues.clone());
    let l1 = (5.0 - 5.0_f64.sqrt()) / 2.0;
    let l2 = (5.0 + 5.0_f64.sqrt()) / 2.0;
    assert!((sv[0] - l1).abs() < 1e-9, "λ₀ = {}", sv[0]);
    assert!((sv[1] - l2).abs() < 1e-9, "λ₁ = {}", sv[1]);
    assert!(orthogonal(&r.eigenvectors, 2, 1e-9));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 2), &m, 1e-9));
    assert!(eigen_property(&m, &r.eigenvectors, &r.eigenvalues, 2, 1e-9));
}

#[test]
fn test_3x3_diagonal() {
    // diag(1, 2, 3)
    let m = vec![
        1.0, 0.0, 0.0,
        0.0, 2.0, 0.0,
        0.0, 0.0, 3.0,
    ];
    let r = symmetric_eigen(&m, 3, 1e-12, 100);
    let sv = sorted(r.eigenvalues.clone());
    assert!((sv[0] - 1.0).abs() < 1e-12);
    assert!((sv[1] - 2.0).abs() < 1e-12);
    assert!((sv[2] - 3.0).abs() < 1e-12);
    assert!(orthogonal(&r.eigenvectors, 3, 1e-12));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 3), &m, 1e-12));
}

#[test]
fn test_3x3_known() {
    // [[2, 0, 0], [0, 3, 4], [0, 4, 9]] — the 2×2 block [[3,4],[4,9]]
    // has eigenvalues 1 and 11, plus the 2.
    let m = vec![
        2.0, 0.0, 0.0,
        0.0, 3.0, 4.0,
        0.0, 4.0, 9.0,
    ];
    let r = symmetric_eigen(&m, 3, 1e-12, 100);
    let sv = sorted(r.eigenvalues.clone());
    assert!((sv[0] - 1.0).abs() < 1e-9, "λ₀ = {}", sv[0]);
    assert!((sv[1] - 2.0).abs() < 1e-9, "λ₁ = {}", sv[1]);
    assert!((sv[2] - 11.0).abs() < 1e-9, "λ₂ = {}", sv[2]);
    assert!(orthogonal(&r.eigenvectors, 3, 1e-9));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 3), &m, 1e-9));
    assert!(eigen_property(&m, &r.eigenvectors, &r.eigenvalues, 3, 1e-9));
}

#[test]
fn test_3x3_dense() {
    // Fully dense symmetric 3×3.
    let m = vec![
        6.0, 2.0, 4.0,
        2.0, 3.0, 5.0,
        4.0, 5.0, 1.0,
    ];
    let r = symmetric_eigen(&m, 3, 1e-12, 100);
    assert_eq!(r.eigenvalues.len(), 3);
    assert_eq!(r.eigenvectors.len(), 9);
    assert!(orthogonal(&r.eigenvectors, 3, 1e-9));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 3), &m, 1e-8));
    assert!(eigen_property(&m, &r.eigenvectors, &r.eigenvalues, 3, 1e-8));
}

#[test]
fn test_4x4_diagonal_scaled() {
    // diag(1, 2, 3, 4)
    let m = vec![
        1.0, 0.0, 0.0, 0.0,
        0.0, 2.0, 0.0, 0.0,
        0.0, 0.0, 3.0, 0.0,
        0.0, 0.0, 0.0, 4.0,
    ];
    let r = symmetric_eigen(&m, 4, 1e-12, 100);
    let sv = sorted(r.eigenvalues.clone());
    for (i, expect) in [1.0, 2.0, 3.0, 4.0].iter().enumerate() {
        assert!((sv[i] - expect).abs() < 1e-12, "λ_{} = {}", i, sv[i]);
    }
    assert!(orthogonal(&r.eigenvectors, 4, 1e-12));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 4), &m, 1e-12));
}

#[test]
fn test_4x4_dense() {
    // Fully dense symmetric 4×4.
    let m = vec![
        10.0, 1.0, 2.0, 3.0,
        1.0, 8.0, 4.0, 5.0,
        2.0, 4.0, 6.0, 7.0,
        3.0, 5.0, 7.0, 4.0,
    ];
    let r = symmetric_eigen(&m, 4, 1e-12, 200);
    assert_eq!(r.eigenvalues.len(), 4);
    assert!(orthogonal(&r.eigenvectors, 4, 1e-9));
    assert!(close(&reconstruct(&r.eigenvectors, &r.eigenvalues, 4), &m, 1e-8));
    assert!(eigen_property(&m, &r.eigenvectors, &r.eigenvalues, 4, 1e-8));
}

#[test]
fn test_empty() {
    let m: Vec<f64> = vec![];
    let r = symmetric_eigen(&m, 0, 1e-12, 100);
    assert!(r.eigenvalues.is_empty());
    assert!(r.eigenvectors.is_empty());
}
