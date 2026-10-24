use matrix_sparse_lu::sparse_lu;

/// Decode a CSR matrix into a dense row-major n×n matrix.
fn csr_decode(values: &[f64], col_idx: &[usize], row_ptr: &[usize], n: usize) -> Vec<f64> {
    let mut dense = vec![0.0; n * n];
    let mut row: usize = 0;
    while row < n {
        let mut p = row_ptr[row];
        while p < row_ptr[row + 1] {
            let col = col_idx[p];
            let idx = row * n + col;
            dense[idx] = values[p];
            p += 1;
        }
        row += 1;
    }
    dense
}

/// Helper — check approximate equality of two f64 values.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = (actual - expected).abs();
    assert!(diff <= tol);
}

/// Multiply two n×n matrices (row-major flat slices).
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

/// Apply the permutation `perm` to the rows of `a` (P * A).
fn apply_perm(a: &[f64], perm: &[usize], n: usize) -> Vec<f64> {
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

/// Verify that P · A = L · U (within tolerance), decoding CSR factors to dense.
fn verify_lu(
    a_values: &[f64],
    a_col_idx: &[usize],
    a_row_ptr: &[usize],
    n: usize,
    tol: f64,
) {
    let result = sparse_lu(a_values, a_col_idx, a_row_ptr, n);
    let a = csr_decode(a_values, a_col_idx, a_row_ptr, n);
    let l = csr_decode(&result.l_values, &result.l_col_idx, &result.l_row_ptr, n);
    let u = csr_decode(&result.u_values, &result.u_col_idx, &result.u_row_ptr, n);
    let lu_prod = matrix_mul(&l, &u, n);
    let pa = apply_perm(&a, &result.perm, n);

    let mut i: usize = 0;
    while i < n * n {
        assert_approx_eq(lu_prod[i], pa[i], tol);
        i += 1;
    }
}

/// Check the triangular structure: L unit lower, U upper.
fn verify_structure(
    a_values: &[f64],
    a_col_idx: &[usize],
    a_row_ptr: &[usize],
    n: usize,
) {
    let result = sparse_lu(a_values, a_col_idx, a_row_ptr, n);
    let l = csr_decode(&result.l_values, &result.l_col_idx, &result.l_row_ptr, n);
    let u = csr_decode(&result.u_values, &result.u_col_idx, &result.u_row_ptr, n);

    let mut i: usize = 0;
    while i < n {
        assert_approx_eq(l[i * n + i], 1.0, 1e-9);
        let mut j: usize = i + 1;
        while j < n {
            assert_approx_eq(l[i * n + j], 0.0, 1e-9);
            j += 1;
        }
        i += 1;
    }
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < i {
            assert_approx_eq(u[i * n + j], 0.0, 1e-9);
            j += 1;
        }
        i += 1;
    }
}

#[test]
fn test_lu_empty() {
    let result = sparse_lu(&[], &[], &[0], 0);
    assert!(result.l_values.len() == 0);
    assert!(result.u_values.len() == 0);
    assert!(result.perm.len() == 0);
    assert!(result.l_row_ptr.len() == 1);
    assert!(result.u_row_ptr.len() == 1);
}

#[test]
fn test_lu_1x1() {
    let values = [5.0];
    let col_idx = [0usize];
    let row_ptr = [0usize, 1usize];
    let result = sparse_lu(&values, &col_idx, &row_ptr, 1);
    assert!(result.perm[0] == 0);
    assert_approx_eq(result.l_values[0], 1.0, 1e-9);
    assert_approx_eq(result.u_values[0], 5.0, 1e-9);
    verify_lu(&values, &col_idx, &row_ptr, 1, 1e-9);
}

#[test]
fn test_lu_2x2_no_pivot() {
    // [[3, 1], [1, 2]] (dense).
    let values = [3.0, 1.0, 1.0, 2.0];
    let col_idx = [0usize, 1usize, 0usize, 1usize];
    let row_ptr = [0usize, 2usize, 4usize];
    let result = sparse_lu(&values, &col_idx, &row_ptr, 2);
    assert!(result.perm[0] == 0);
    assert!(result.perm[1] == 1);
    verify_lu(&values, &col_idx, &row_ptr, 2, 1e-9);
}

#[test]
fn test_lu_2x2_with_pivot() {
    // [[2, 1], [4, 3]] — row swap required.
    let values = [2.0, 1.0, 4.0, 3.0];
    let col_idx = [0usize, 1usize, 0usize, 1usize];
    let row_ptr = [0usize, 2usize, 4usize];
    let result = sparse_lu(&values, &col_idx, &row_ptr, 2);
    assert!(result.perm[0] == 1);
    assert!(result.perm[1] == 0);
    verify_lu(&values, &col_idx, &row_ptr, 2, 1e-9);
}

#[test]
fn test_lu_3x3_sparse() {
    // Sparse invertible matrix with structural zeros.
    // [[0, 1, 2], [3, 4, 5], [6, 7, 0]]
    // row 0: (0,1)=1, (0,2)=2
    // row 1: (1,0)=3, (1,1)=4, (1,2)=5
    // row 2: (2,0)=6, (2,1)=7
    let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let col_idx = [1usize, 2usize, 0usize, 1usize, 2usize, 0usize, 1usize];
    let row_ptr = [0usize, 2usize, 5usize, 7usize];
    verify_lu(&values, &col_idx, &row_ptr, 3, 1e-9);
    verify_structure(&values, &col_idx, &row_ptr, 3);
}

#[test]
fn test_lu_3x3_identity() {
    let values = [1.0, 1.0, 1.0];
    let col_idx = [0usize, 1usize, 2usize];
    let row_ptr = [0usize, 1usize, 2usize, 3usize];
    let result = sparse_lu(&values, &col_idx, &row_ptr, 3);
    assert!(result.perm[0] == 0);
    assert!(result.perm[1] == 1);
    assert!(result.perm[2] == 2);
    verify_lu(&values, &col_idx, &row_ptr, 3, 1e-9);
}

#[test]
fn test_lu_3x3_zeros_first_col() {
    // [[0,1,2],[0,3,4],[5,6,7]] — pivoting needed.
    let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let col_idx = [1usize, 2usize, 1usize, 2usize, 0usize, 1usize, 2usize];
    let row_ptr = [0usize, 2usize, 4usize, 7usize];
    let result = sparse_lu(&values, &col_idx, &row_ptr, 3);
    assert!(result.perm[0] == 2);
    verify_lu(&values, &col_idx, &row_ptr, 3, 1e-9);
}

#[test]
fn test_lu_4x4_diagonal() {
    // [[2,0,0,0],[0,3,0,0],[0,0,5,0],[0,0,0,7]]
    let values = [2.0, 3.0, 5.0, 7.0];
    let col_idx = [0usize, 1usize, 2usize, 3usize];
    let row_ptr = [0usize, 1usize, 2usize, 3usize, 4usize];
    verify_lu(&values, &col_idx, &row_ptr, 4, 1e-9);
    verify_structure(&values, &col_idx, &row_ptr, 4);
}

#[test]
fn test_lu_4x4_general() {
    // [[1,0,2,-1],[3,0,0,5],[2,1,4,-3],[1,0,5,0]]
    let values = [1.0, 2.0, -1.0, 3.0, 5.0, 2.0, 1.0, 4.0, -3.0, 1.0, 5.0];
    let col_idx = [0usize, 2usize, 3usize, 0usize, 3usize, 0usize, 1usize, 2usize, 3usize, 0usize, 2usize];
    let row_ptr = [0usize, 3usize, 5usize, 9usize, 11usize];
    verify_lu(&values, &col_idx, &row_ptr, 4, 1e-9);
}

#[test]
#[should_panic(expected = "singular matrix")]
fn test_lu_2x2_singular() {
    // [[1, 2], [2, 4]] — linearly dependent rows.
    let values = [1.0, 2.0, 2.0, 4.0];
    let col_idx = [0usize, 1usize, 0usize, 1usize];
    let row_ptr = [0usize, 2usize, 4usize];
    let _ = sparse_lu(&values, &col_idx, &row_ptr, 2);
}
