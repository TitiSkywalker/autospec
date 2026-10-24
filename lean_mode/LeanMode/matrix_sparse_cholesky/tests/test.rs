//! Rust-side verification of the sparse Cholesky factorization.
//!
//! This file is *not* translated (the `Real` float-model skips tests); it is
//! run by `cargo test` only.  Idiomatic Rust is therefore fine here.

use matrix_sparse_cholesky::sparse_cholesky;

/// Decode a CSR matrix (full symmetric) into a dense n×n matrix.
fn csr_decode(values: &[f64], col_idx: &[usize], row_ptr: &[usize], n: usize) -> Vec<f64> {
    let mut dense = vec![0.0; n * n];
    for i in 0..n {
        for p in row_ptr[i]..row_ptr[i + 1] {
            dense[i * n + col_idx[p]] = values[p];
        }
    }
    dense
}

fn at(m: &[f64], n: usize, row: usize, col: usize) -> f64 {
    m[row * n + col]
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

/// Verify A = L·Lᵀ, L lower triangular, and L has a positive diagonal.
fn check(a_values: &[f64], a_col_idx: &[usize], a_row_ptr: &[usize], n: usize) {
    let a = csr_decode(a_values, a_col_idx, a_row_ptr, n);
    let result = sparse_cholesky(a_values, a_col_idx, a_row_ptr, n);
    let l = csr_decode(&result.l_values, &result.l_col_idx, &result.l_row_ptr, n);

    // Lower triangular + positive diagonal.
    for i in 0..n {
        assert!(at(&l, n, i, i) > 0.0, "diagonal entry L[{},{}] must be > 0", i, i);
        for j in (i + 1)..n {
            assert!(
                at(&l, n, i, j).abs() < 1e-9,
                "L[{},{}] should be 0 (strict upper triangle)",
                i,
                j
            );
        }
    }

    // Reconstruct A = L·Lᵀ and compare.
    let mut a2 = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += at(&l, n, i, k) * at(&l, n, j, k);
            }
            a2[i * n + j] = sum;
        }
    }
    assert!(close(&a, &a2, 1e-9), "L·Lᵀ does not match A");
}

#[test]
fn test_empty() {
    let result = sparse_cholesky(&[], &[], &[0], 0);
    assert!(result.l_values.is_empty());
    assert!(result.l_col_idx.is_empty());
    assert_eq!(result.l_row_ptr.len(), 1);
    assert_eq!(result.l_row_ptr[0], 0);
}

#[test]
fn test_1x1() {
    let values = [4.0];
    let col_idx = [0usize];
    let row_ptr = [0usize, 1usize];
    let result = sparse_cholesky(&values, &col_idx, &row_ptr, 1);
    // L = [[2]]
    assert!((result.l_values[0] - 2.0).abs() < 1e-9);
    assert_eq!(result.l_col_idx[0], 0);
    assert_eq!(result.l_row_ptr, vec![0, 1]);
}

#[test]
fn test_2x2_spd() {
    // [[4, 2], [2, 3]] — dense full symmetric matrix.
    let values = [4.0, 2.0, 2.0, 3.0];
    let col_idx = [0usize, 1usize, 0usize, 1usize];
    let row_ptr = [0usize, 2usize, 4usize];
    check(&values, &col_idx, &row_ptr, 2);
}

#[test]
fn test_3x3_dense_lower() {
    // SPD matrix [[4,2,1],[2,3,1],[1,1,2]] — full symmetric matrix.
    let values = [4.0, 2.0, 1.0, 2.0, 3.0, 1.0, 1.0, 1.0, 2.0];
    let col_idx = [0usize, 1usize, 2usize, 0usize, 1usize, 2usize, 0usize, 1usize, 2usize];
    let row_ptr = [0usize, 3usize, 6usize, 9usize];
    check(&values, &col_idx, &row_ptr, 3);
}

#[test]
fn test_3x3_diagonal() {
    // [[9, 0, 0], [0, 16, 0], [0, 0, 25]] — diagonal SPD.
    let values = [9.0, 16.0, 25.0];
    let col_idx = [0usize, 1usize, 2usize];
    let row_ptr = [0usize, 1usize, 2usize, 3usize];
    let result = sparse_cholesky(&values, &col_idx, &row_ptr, 3);
    // L = diag(3, 4, 5) — no off-diagonal non-zeros.
    let l = csr_decode(&result.l_values, &result.l_col_idx, &result.l_row_ptr, 3);
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { (3 + i) as f64 } else { 0.0 };
            assert!((at(&l, 3, i, j) - expected).abs() < 1e-9);
        }
    }
}

#[test]
fn test_4x4_with_fill() {
    // SPD matrix (full symmetric) whose Cholesky factor exhibits fill-in:
    //
    //   A = [[4, 0, 1, 0],
    //        [0, 4, 1, 0],
    //        [1, 1, 4, 1],
    //        [0, 0, 1, 4]]
    //
    // Nodes 0,1,2,3 with edges (0,2),(1,2),(2,3).  Eliminating 0 against 2
    // introduces fill at L[3,0] even though A[3,0] = 0; eliminating 1 against 2
    // introduces fill at L[3,1] even though A[3,1] = 0.
    let values = [4.0, 1.0, 4.0, 1.0, 1.0, 1.0, 4.0, 1.0, 1.0, 4.0];
    let col_idx = [0usize, 2usize, 1usize, 2usize, 0usize, 1usize, 2usize, 3usize, 2usize, 3usize];
    let row_ptr = [0usize, 2usize, 4usize, 8usize, 10usize];
    check(&values, &col_idx, &row_ptr, 4);
}

#[test]
fn test_4x4_dense_block_fill_regression() {
    // Regression test for the symbolic-fill sentinel collision: a 3×3 dense
    // block plus a decoupled diagonal, which makes the fill pattern grow past
    // `n` nonzeros (so a global-index "present" marker would collide with the
    // `n` sentinel and duplicate rows).
    //
    //   A = [[4, 1, 1, 0],
    //        [1, 4, 1, 0],
    //        [1, 1, 4, 0],
    //        [0, 0, 0, 4]]
    let values = [4.0, 1.0, 1.0, 1.0, 4.0, 1.0, 1.0, 1.0, 4.0, 4.0];
    let col_idx = [0usize, 1usize, 2usize, 0usize, 1usize, 2usize, 0usize, 1usize, 2usize, 3usize];
    let row_ptr = [0usize, 3usize, 6usize, 9usize, 10usize];
    check(&values, &col_idx, &row_ptr, 4);
}

#[test]
#[should_panic(expected = "not positive-definite")]
fn test_not_positive_definite() {
    // [[1, 0], [0, -1]] — diagonal pivot becomes negative.
    let values = [1.0, -1.0];
    let col_idx = [0usize, 1usize];
    let row_ptr = [0usize, 1usize, 2usize];
    let _ = sparse_cholesky(&values, &col_idx, &row_ptr, 2);
}
