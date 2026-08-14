use matrix_svd::*;

/// Reconstruct A = U · Σ · Vᵀ (row-major, k = min(m, n)).
fn reconstruct(u: &[f64], s: &[f64], v_t: &[f64], m: usize, n: usize, k: usize) -> Vec<f64> {
    let mut out = vec![0.0; m * n];
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0;
            for l in 0..k {
                sum += u[i * k + l] * s[l] * v_t[l * n + j];
            }
            out[i * n + j] = sum;
        }
    }
    out
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

/// Check Uᵀ U = I (columns of U, a `rows`×`cols` matrix, are orthonormal).
fn orthonormal_cols(u: &[f64], rows: usize, cols: usize, tol: f64) -> bool {
    for i in 0..cols {
        for j in 0..cols {
            let mut dot = 0.0;
            for r in 0..rows {
                dot += u[r * cols + i] * u[r * cols + j];
            }
            let expect = if i == j { 1.0 } else { 0.0 };
            if (dot - expect).abs() > tol {
                return false;
            }
        }
    }
    true
}

/// Check Vᵀ · V = I (rows of Vᵀ, a `k`×`n` matrix, are orthonormal).
fn orthonormal_rows(v_t: &[f64], k: usize, n: usize, tol: f64) -> bool {
    for i in 0..k {
        for j in 0..k {
            let mut dot = 0.0;
            for c in 0..n {
                dot += v_t[i * n + c] * v_t[j * n + c];
            }
            let expect = if i == j { 1.0 } else { 0.0 };
            if (dot - expect).abs() > tol {
                return false;
            }
        }
    }
    true
}

fn sorted(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

#[test]
fn test_diagonal_2x2() {
    // [[2, 0], [0, 3]] in row-major order: row0 = [2,0], row1 = [0,3].
    let m = vec![2.0, 0.0, 0.0, 3.0];
    let r = svd(&m, 2, 2, 1e-12, 100);

    assert_eq!(r.singular_values.len(), 2);
    assert_eq!(r.u.len(), 4);
    assert_eq!(r.v_t.len(), 4);

    let sv = sorted(r.singular_values.clone());
    assert!((sv[0] - 2.0).abs() < 1e-9, "σ₀ = {}", sv[0]);
    assert!((sv[1] - 3.0).abs() < 1e-9, "σ₁ = {}", sv[1]);

    assert!(orthonormal_cols(&r.u, 2, 2, 1e-9));
    assert!(orthonormal_rows(&r.v_t, 2, 2, 1e-9));
    assert!(close(&reconstruct(&r.u, &r.singular_values, &r.v_t, 2, 2, 2), &m, 1e-9));
}

#[test]
fn test_diagonal_negative_entry() {
    // [[3, 0], [0, -4]] — singular values are the absolute diagonal entries.
    let m = vec![3.0, 0.0, 0.0, -4.0];
    let r = svd(&m, 2, 2, 1e-12, 100);

    let sv = sorted(r.singular_values.clone());
    assert!((sv[0] - 3.0).abs() < 1e-9, "σ₀ = {}", sv[0]);
    assert!((sv[1] - 4.0).abs() < 1e-9, "σ₁ = {}", sv[1]);

    assert!(orthonormal_cols(&r.u, 2, 2, 1e-9));
    assert!(orthonormal_rows(&r.v_t, 2, 2, 1e-9));
    assert!(close(&reconstruct(&r.u, &r.singular_values, &r.v_t, 2, 2, 2), &m, 1e-9));
}

#[test]
fn test_nondiagonal_2x2() {
    // [[1, 1], [0, 1]] — singular values are φ ≈ 1.618 and 1/φ ≈ 0.618.
    let m = vec![1.0, 1.0, 0.0, 1.0];
    let r = svd(&m, 2, 2, 1e-12, 100);

    let sv = sorted(r.singular_values.clone());
    assert!((sv[0] - 0.6180339887).abs() < 1e-6, "σ₀ = {}", sv[0]);
    assert!((sv[1] - 1.6180339887).abs() < 1e-6, "σ₁ = {}", sv[1]);

    assert!(orthonormal_cols(&r.u, 2, 2, 1e-9));
    assert!(orthonormal_rows(&r.v_t, 2, 2, 1e-9));
    assert!(close(&reconstruct(&r.u, &r.singular_values, &r.v_t, 2, 2, 2), &m, 1e-9));
}

#[test]
fn test_tall_3x2() {
    // [[1, 4], [2, 5], [3, 6]] (3 rows, 2 cols) row-major.
    // Singular values are √((91 ± √8065)/2) ≈ 0.7728 and 9.5080.
    let m = vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0];
    let r = svd(&m, 3, 2, 1e-12, 100);

    assert_eq!(r.singular_values.len(), 2);
    assert_eq!(r.u.len(), 6); // 3×2
    assert_eq!(r.v_t.len(), 4); // 2×2

    let sv = sorted(r.singular_values.clone());
    assert!((sv[0] - 0.77283).abs() < 1e-4, "σ₀ = {}", sv[0]);
    assert!((sv[1] - 9.50803).abs() < 1e-4, "σ₁ = {}", sv[1]);

    assert!(orthonormal_cols(&r.u, 3, 2, 1e-9));
    assert!(orthonormal_rows(&r.v_t, 2, 2, 1e-9));
    assert!(close(&reconstruct(&r.u, &r.singular_values, &r.v_t, 3, 2, 2), &m, 1e-9));
}

#[test]
fn test_wide_2x3() {
    // [[1, 2, 3], [4, 5, 6]] (2 rows, 3 cols) row-major.
    // Exercises the m < n branch (transpose → Jacobi → swap factors).
    let m = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let r = svd(&m, 2, 3, 1e-12, 100);

    assert_eq!(r.singular_values.len(), 2);
    assert_eq!(r.u.len(), 4); // 2×2
    assert_eq!(r.v_t.len(), 6); // 2×3

    // Same nonzero singular values as the 3×2 transpose.
    let sv = sorted(r.singular_values.clone());
    assert!((sv[0] - 0.77283).abs() < 1e-4, "σ₀ = {}", sv[0]);
    assert!((sv[1] - 9.50803).abs() < 1e-4, "σ₁ = {}", sv[1]);

    assert!(orthonormal_cols(&r.u, 2, 2, 1e-9));
    assert!(orthonormal_rows(&r.v_t, 2, 3, 1e-9));
    assert!(close(&reconstruct(&r.u, &r.singular_values, &r.v_t, 2, 3, 2), &m, 1e-9));
}

#[test]
fn test_4x4_identity_scaled() {
    // diag(1, 2, 3, 4) — singular values {1,2,3,4}.
    let m = vec![
        1.0, 0.0, 0.0, 0.0,
        0.0, 2.0, 0.0, 0.0,
        0.0, 0.0, 3.0, 0.0,
        0.0, 0.0, 0.0, 4.0,
    ];
    let r = svd(&m, 4, 4, 1e-12, 100);

    let sv = sorted(r.singular_values.clone());
    assert!((sv[0] - 1.0).abs() < 1e-9, "σ₀ = {}", sv[0]);
    assert!((sv[1] - 2.0).abs() < 1e-9, "σ₁ = {}", sv[1]);
    assert!((sv[2] - 3.0).abs() < 1e-9, "σ₂ = {}", sv[2]);
    assert!((sv[3] - 4.0).abs() < 1e-9, "σ₃ = {}", sv[3]);

    assert!(orthonormal_cols(&r.u, 4, 4, 1e-9));
    assert!(orthonormal_rows(&r.v_t, 4, 4, 1e-9));
    assert!(close(&reconstruct(&r.u, &r.singular_values, &r.v_t, 4, 4, 4), &m, 1e-9));
}

#[test]
fn test_empty() {
    let m: Vec<f64> = vec![];
    let r = svd(&m, 0, 0, 1e-12, 100);
    assert!(r.singular_values.is_empty());
    assert!(r.u.is_empty());
    assert!(r.v_t.is_empty());
}
