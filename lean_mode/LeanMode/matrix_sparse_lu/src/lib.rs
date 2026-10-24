//! Sparse LU factorization `P · A = L · U` with partial (row) pivoting.
//!
//! The input `A` is given in CSR (Compressed Sparse Row) form and decoded into
//! a dense n×n working matrix; Gaussian elimination with partial pivoting then
//! performs a *sparse right-looking* update — the elimination only touches
//! rows `i` with a non-zero multiplier and columns `j` of the pivot row with a
//! non-zero entry (skip-when-zero is exactly the sparse-elimination loop).  The
//! factors `L` (unit lower-triangular) and `U` (upper-triangular) are returned
//! in CSR form, together with the row permutation.
//!
//! ## CSR layout
//!
//! `row_ptr` has length `n+1`; the non-zeros of row `i` live in
//! `values[row_ptr[i] .. row_ptr[i+1]]` with columns
//! `col_idx[row_ptr[i] .. row_ptr[i+1]]`.  `A` may be any square matrix (the
//! non-zeros can be stored in any order, but each `(i, j)` at most once).

/// Result of the sparse LU factorization.
pub struct SparseLU {
    /// Unit lower-triangular factor L (CSR).
    pub l_values: Vec<f64>,
    pub l_col_idx: Vec<usize>,
    pub l_row_ptr: Vec<usize>,
    /// Upper-triangular factor U (CSR).
    pub u_values: Vec<f64>,
    pub u_col_idx: Vec<usize>,
    pub u_row_ptr: Vec<usize>,
    /// Permutation: `perm[i]` is the original row that now sits at row i.
    pub perm: Vec<usize>,
}

/// Decode a CSR matrix into a dense row-major n×n matrix.
fn csr_to_dense(
    values: &[f64],
    col_idx: &[usize],
    row_ptr: &[usize],
    n: usize,
) -> Vec<f64> {
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

/// Encode the unit lower-triangular factor L in CSR form.
///
/// The diagonal is always stored (value 1.0); a strictly-lower entry is stored
/// iff it is non-zero.
fn encode_l(lu: &[f64], n: usize) -> (Vec<f64>, Vec<usize>, Vec<usize>) {
    let mut l_values: Vec<f64> = Vec::new();
    let mut l_col_idx: Vec<usize> = Vec::new();
    let mut l_row_ptr = vec![0usize; n + 1];

    let mut row: usize = 0;
    while row < n {
        // Unit diagonal.
        l_values.push(1.0);
        l_col_idx.push(row);

        // Strictly-lower entries.
        let mut col: usize = 0;
        while col < row {
            let v = lu[row * n + col];
            if v != 0.0 {
                l_values.push(v);
                l_col_idx.push(col);
            }
            col += 1;
        }

        let len = l_values.len();
        l_row_ptr[row + 1] = len;
        row += 1;
    }

    (l_values, l_col_idx, l_row_ptr)
}

/// Encode the upper-triangular factor U in CSR form (store entries i ≤ j that
/// are non-zero).
fn encode_u(lu: &[f64], n: usize) -> (Vec<f64>, Vec<usize>, Vec<usize>) {
    let mut u_values: Vec<f64> = Vec::new();
    let mut u_col_idx: Vec<usize> = Vec::new();
    let mut u_row_ptr = vec![0usize; n + 1];

    let mut row: usize = 0;
    while row < n {
        let mut col: usize = row;
        while col < n {
            let v = lu[row * n + col];
            if v != 0.0 {
                u_values.push(v);
                u_col_idx.push(col);
            }
            col += 1;
        }

        let len = u_values.len();
        u_row_ptr[row + 1] = len;
        row += 1;
    }

    (u_values, u_col_idx, u_row_ptr)
}

/// Compute the sparse LU factorization of an n×n matrix `A` given in CSR form.
///
/// Returns `SparseLU` such that `P · csr_decode(A) = csr_decode(L) · csr_decode(U)`,
/// with `L` unit lower-triangular and `U` upper-triangular.
///
/// # Panics
///
/// Panics if a zero pivot is encountered (the matrix is singular).
pub fn sparse_lu(
    a_values: &[f64],
    a_col_idx: &[usize],
    a_row_ptr: &[usize],
    n: usize,
) -> SparseLU {
    if n == 0 {
        let mut l_row_ptr: Vec<usize> = Vec::new();
        l_row_ptr.push(0);
        let mut u_row_ptr: Vec<usize> = Vec::new();
        u_row_ptr.push(0);
        return SparseLU {
            l_values: Vec::new(),
            l_col_idx: Vec::new(),
            l_row_ptr,
            u_values: Vec::new(),
            u_col_idx: Vec::new(),
            u_row_ptr,
            perm: Vec::new(),
        };
    }

    // 1. Decode CSR -> dense working matrix (row-major).
    let mut lu = csr_to_dense(a_values, a_col_idx, a_row_ptr, n);

    // 2. Initial permutation = identity.
    let mut perm: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < n {
        perm.push(i);
        i += 1;
    }

    // 3. Gaussian elimination with partial pivoting (sparse right-looking).
    let mut k: usize = 0;
    while k < n {
        // --- partial pivoting: largest |value| in column k, rows ≥ k ---
        let mut max_abs = lu[k * n + k];
        if max_abs < 0.0 {
            max_abs = -max_abs;
        }
        let mut piv = k;

        let mut i: usize = k + 1;
        while i < n {
            let mut abs_val = lu[i * n + k];
            if abs_val < 0.0 {
                abs_val = -abs_val;
            }
            if abs_val > max_abs {
                max_abs = abs_val;
                piv = i;
            }
            i += 1;
        }

        let diag = lu[piv * n + k];
        assert!(diag != 0.0, "singular matrix");

        // --- row swap (if needed) ---
        if piv != k {
            let tmp_perm = perm[k];
            perm[k] = perm[piv];
            perm[piv] = tmp_perm;

            let mut j: usize = 0;
            while j < n {
                let tmp = lu[k * n + j];
                lu[k * n + j] = lu[piv * n + j];
                lu[piv * n + j] = tmp;
                j += 1;
            }
        }

        // --- sparse elimination: skip zero multipliers and zero pivot-row cols ---
        let mut i: usize = k + 1;
        while i < n {
            let a_ik = lu[i * n + k];
            if a_ik != 0.0 {
                let mult = a_ik / diag;
                lu[i * n + k] = mult;

                let mut j: usize = k + 1;
                while j < n {
                    let a_kj = lu[k * n + j];
                    if a_kj != 0.0 {
                        let prod = mult * a_kj;
                        let new_val = lu[i * n + j] - prod;
                        lu[i * n + j] = new_val;
                    }
                    j += 1;
                }
            }
            i += 1;
        }

        k += 1;
    }

    // 4. Encode L and U in CSR form.
    let (l_values, l_col_idx, l_row_ptr) = encode_l(&lu, n);
    let (u_values, u_col_idx, u_row_ptr) = encode_u(&lu, n);

    SparseLU {
        l_values,
        l_col_idx,
        l_row_ptr,
        u_values,
        u_col_idx,
        u_row_ptr,
        perm,
    }
}
