use matrix_bidiagonal::{bidiagonal, bidiagonal_decompose};

/// Test: simple 2×2 matrix
#[test]
fn test_simple_2x2() {
    let mut matrix = vec![1.0, 2.0, 3.0, 4.0]; // col-major: [[1,3],[2,4]]
    let original = matrix.clone();
    let (U, B, Vt) = bidiagonal(&mut matrix, 2, 2);
    let k = 2;
    assert_eq!(U.len(), 2 * k);  // m × k
    assert_eq!(B.len(), k * k);  // k × k
    assert_eq!(Vt.len(), k * 2); // k × n
    // Verify U^T U = I_k
    check_orthogonal_rect(&U, 2, k);
    // Verify Vt Vt^T = I_k
    check_vt_orthogonal(&Vt, k, 2);
    // Verify M = U * B * Vt
    check_decomposition(&original, &U, &B, &Vt, 2, 2, k);
}

/// Test: identity matrix 2×2
#[test]
fn test_identity_2x2() {
    let mut matrix = vec![1.0, 0.0, 0.0, 1.0];
    let original = matrix.clone();
    let (U, B, Vt) = bidiagonal(&mut matrix, 2, 2);
    let k = 2;
    check_orthogonal_rect(&U, 2, k);
    check_vt_orthogonal(&Vt, k, 2);
    check_decomposition(&original, &U, &B, &Vt, 2, 2, k);
}

/// Test: rectangular 3×2 (tall)
#[test]
fn test_tall_3x2() {
    let mut matrix = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let original = matrix.clone();
    let (U, B, Vt) = bidiagonal(&mut matrix, 3, 2);
    let k = 2;
    assert_eq!(U.len(), 3 * k);  // 3×2
    assert_eq!(B.len(), k * k);  // 2×2
    assert_eq!(Vt.len(), k * 2); // 2×2
    check_orthogonal_rect(&U, 3, k);
    check_vt_orthogonal(&Vt, k, 2);
    check_decomposition(&original, &U, &B, &Vt, 3, 2, k);
}

/// Test: rectangular 2×3 (wide)
#[test]
fn test_wide_2x3() {
    let mut matrix = vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]; // [[1,2,3],[4,5,6]]
    let original = matrix.clone();
    let (U, B, Vt) = bidiagonal(&mut matrix, 2, 3);
    let k = 2;
    assert_eq!(U.len(), 2 * k);   // 2×2
    assert_eq!(B.len(), k * k);   // 2×2
    assert_eq!(Vt.len(), k * 3);  // 2×3
    check_orthogonal_rect(&U, 2, k);
    check_vt_orthogonal(&Vt, k, 3);
    check_decomposition(&original, &U, &B, &Vt, 2, 3, k);
}

/// Test: 1×1 matrix
#[test]
fn test_single_element() {
    let mut matrix = vec![42.0];
    let original = matrix.clone();
    let (U, B, Vt) = bidiagonal(&mut matrix, 1, 1);
    let k = 1;
    assert_eq!(U.len(), 1);
    assert_eq!(B.len(), 1);
    assert_eq!(Vt.len(), 1);
    check_orthogonal_rect(&U, 1, k);
    check_vt_orthogonal(&Vt, k, 1);
    check_decomposition(&original, &U, &B, &Vt, 1, 1, k);
}

/// Test: 3×3 matrix
#[test]
fn test_square_3x3() {
    let mut matrix = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let original = matrix.clone();
    let (U, B, Vt) = bidiagonal(&mut matrix, 3, 3);
    let k = 3;
    check_orthogonal_rect(&U, 3, k);
    check_vt_orthogonal(&Vt, k, 3);
    check_decomposition(&original, &U, &B, &Vt, 3, 3, k);
}

/// Test: pre-allocated workspace gives consistent diagonal/off_diagonal
#[test]
fn test_preallocated_matches() {
    let mut mat1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 3×2
    let mut mat2 = mat1.clone();

    let (_, B_w, _) = bidiagonal(&mut mat1, 3, 2);

    let mut diagonal = vec![0.0; 2];
    let mut off_diagonal = vec![0.0; 1];
    let mut col_axis = vec![0.0; 3];
    let mut row_axis = vec![0.0; 2];
    let mut work = vec![0.0; 3];
    let mut sub = vec![0.0; 6];

    let up = bidiagonal_decompose(
        &mut mat2, 3, 2,
        &mut diagonal, &mut off_diagonal,
        &mut col_axis, &mut row_axis, &mut work, &mut sub,
    );
    assert!(up);

    let mut i: usize = 0;
    while i < diagonal.len() {
        let diff = B_w[i + i * 2] - diagonal[i].abs();
        let abs_diff = if diff < 0.0 { -diff } else { diff };
        assert!(abs_diff < 1e-10);
        i = i + 1;
    }
    let mut j: usize = 0;
    while j < off_diagonal.len() {
        let diff = B_w[j + (j + 1) * 2] - off_diagonal[j].abs();
        let abs_diff = if diff < 0.0 { -diff } else { diff };
        assert!(abs_diff < 1e-10);
        j = j + 1;
    }
}

// ── verification helpers ──────────────────────────────────────────────────────

/// Check that `U` (m × k) has orthonormal columns: U^T U = I_k.
fn check_orthogonal_rect(U: &[f64], m: usize, k: usize) {
    let mut i: usize = 0;
    while i < k {
        let mut j: usize = 0;
        while j < k {
            // (U^T U)[i,j] = sum_t U[t + i*m] * U[t + j*m]
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < m {
                dot = dot + U[t + i * m] * U[t + j * m];
                t = t + 1;
            }
            let expected = if i == j { 1.0 } else { 0.0 };
            let diff = dot - expected;
            let abs_diff = if diff < 0.0 { -diff } else { diff };
            assert!(
                abs_diff < 1e-10,
                "U^T U[{i}][{j}] = {dot}, expected {expected}"
            );
            j = j + 1;
        }
        i = i + 1;
    }
}

/// Check that Vt (k × n) has orthonormal rows: Vt Vt^T = I_k.
fn check_vt_orthogonal(Vt: &[f64], k: usize, n: usize) {
    let mut i: usize = 0;
    while i < k {
        let mut j: usize = 0;
        while j < k {
            // (Vt Vt^T)[i,j] = sum_t Vt[i + t*k] * Vt[j + t*k]  (column-major Vt)
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < n {
                dot = dot + Vt[i + t * k] * Vt[j + t * k];
                t = t + 1;
            }
            let expected = if i == j { 1.0 } else { 0.0 };
            let diff = dot - expected;
            let abs_diff = if diff < 0.0 { -diff } else { diff };
            assert!(
                abs_diff < 1e-10,
                "Vt Vt^T[{i}][{j}] = {dot}, expected {expected}"
            );
            j = j + 1;
        }
        i = i + 1;
    }
}

/// Check that M = U * B * Vt  (M: m×n, U: m×k, B: k×k, Vt: k×n, all column-major).
fn check_decomposition(M: &[f64], U: &[f64], B: &[f64], Vt: &[f64], m: usize, n: usize, k: usize) {
    // Compute BVt  (size k×n)
    let mut BVt = vec![0.0; k * n];
    let mut col: usize = 0;
    while col < n {
        let mut row: usize = 0;
        while row < k {
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < k {
                dot = dot + B[row + t * k] * Vt[t + col * k];
                t = t + 1;
            }
            BVt[row + col * k] = dot;
            row = row + 1;
        }
        col = col + 1;
    }

    // Compute U * BVt  (size m×n)
    let mut UBVt = vec![0.0; m * n];
    let mut col: usize = 0;
    while col < n {
        let mut row: usize = 0;
        while row < m {
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < k {
                dot = dot + U[row + t * m] * BVt[t + col * k];
                t = t + 1;
            }
            UBVt[row + col * m] = dot;
            row = row + 1;
        }
        col = col + 1;
    }

    // Compare with M
    let mut idx: usize = 0;
    while idx < m * n {
        let diff = UBVt[idx] - M[idx];
        let abs_diff = if diff < 0.0 { -diff } else { diff };
        assert!(
            abs_diff < 1e-9,
            "(U B Vt)[{idx}] = {}, M[{idx}] = {}", UBVt[idx], M[idx]
        );
        idx = idx + 1;
    }
}
