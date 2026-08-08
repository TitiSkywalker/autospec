use matrix_bidiagonal::{bidiagonal, bidiagonal_decompose};

/// Test: simple 2×2 matrix
#[test]
fn test_simple_2x2() {
    let mut matrix = vec![1.0, 2.0, 3.0, 4.0]; // col-major: [[1,3],[2,4]]
    let (diagonal, off_diagonal, upper_diagonal, _uv) =
        bidiagonal(&mut matrix, 2, 2);
    assert!(upper_diagonal);
    assert_eq!(diagonal.len(), 2);
    assert_eq!(off_diagonal.len(), 1);
    assert!(!diagonal[0].is_nan());
    assert!(!diagonal[1].is_nan());
    assert!(!off_diagonal[0].is_nan());
}

/// Test: identity matrix 2×2
#[test]
fn test_identity_2x2() {
    let mut matrix = vec![1.0, 0.0, 0.0, 1.0];
    let (diagonal, off_diagonal, upper_diagonal, _uv) =
        bidiagonal(&mut matrix, 2, 2);
    assert!(upper_diagonal);
    assert!(diagonal[0].abs() > 0.5);
    assert!(diagonal[1].abs() > 0.5);
    assert!(off_diagonal[0].abs() < 0.1);
}

/// Test: rectangular 3×2 (tall)
#[test]
fn test_tall_3x2() {
    let mut matrix = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let (diagonal, off_diagonal, upper_diagonal, _uv) =
        bidiagonal(&mut matrix, 3, 2);
    assert!(upper_diagonal);
    assert_eq!(diagonal.len(), 2);
    assert_eq!(off_diagonal.len(), 1);
    assert!(!diagonal[0].is_nan());
    assert!(!diagonal[1].is_nan());
    assert!(!off_diagonal[0].is_nan());
}

/// Test: rectangular 2×3 (wide)
#[test]
fn test_wide_2x3() {
    let mut matrix = vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]; // [[1,2,3],[4,5,6]]
    let (diagonal, off_diagonal, upper_diagonal, _uv) =
        bidiagonal(&mut matrix, 2, 3);
    assert!(!upper_diagonal);
    assert_eq!(diagonal.len(), 2);
    assert_eq!(off_diagonal.len(), 1);
    assert!(!diagonal[0].is_nan() && !diagonal[0].is_infinite());
    assert!(!diagonal[1].is_nan() && !diagonal[1].is_infinite());
    assert!(!off_diagonal[0].is_nan() && !off_diagonal[0].is_infinite());
    assert!(diagonal[0].abs() > 0.0);
    assert!(diagonal[1].abs() > 0.0);
}

/// Test: 1×1 matrix
#[test]
fn test_single_element() {
    let mut matrix = vec![42.0];
    let (diagonal, off_diagonal, upper_diagonal, _uv) =
        bidiagonal(&mut matrix, 1, 1);
    assert!(upper_diagonal);
    assert_eq!(diagonal.len(), 1);
    assert_eq!(off_diagonal.len(), 0);
    assert!(diagonal[0].abs() > 40.0);
}

/// Test: 3×3 matrix
#[test]
fn test_square_3x3() {
    let mut matrix = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    ];
    let (diagonal, off_diagonal, upper_diagonal, _uv) =
        bidiagonal(&mut matrix, 3, 3);
    assert!(upper_diagonal);
    assert_eq!(diagonal.len(), 3);
    assert_eq!(off_diagonal.len(), 2);
    let mut idx: usize = 0;
    while idx < diagonal.len() {
        assert!(!diagonal[idx].is_nan());
        idx = idx + 1;
    }
    let mut idx2: usize = 0;
    while idx2 < off_diagonal.len() {
        assert!(!off_diagonal[idx2].is_nan());
        idx2 = idx2 + 1;
    }
}

/// Test: pre-allocated workspace gives same result
#[test]
fn test_preallocated_matches() {
    let mut mat1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 3×2
    let mut mat2 = mat1.clone();

    let (diag_w, off_w, up_w, _uv_w) = bidiagonal(&mut mat1, 3, 2);

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

    assert_eq!(up_w, up);
    let mut i: usize = 0;
    while i < diag_w.len() {
        let diff = diag_w[i] - diagonal[i];
        let abs_diff = if diff < 0.0 { -diff } else { diff };
        assert!(abs_diff < 1e-10);
        i = i + 1;
    }
    let mut j: usize = 0;
    while j < off_w.len() {
        let diff = off_w[j] - off_diagonal[j];
        let abs_diff = if diff < 0.0 { -diff } else { diff };
        assert!(abs_diff < 1e-10);
        j = j + 1;
    }
}
