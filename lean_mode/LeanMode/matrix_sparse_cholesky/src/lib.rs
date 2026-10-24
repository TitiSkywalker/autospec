//! Sparse Cholesky factorization `A = L · Lᵀ` for a symmetric positive-definite
//! matrix `A` given in CSR (Compressed Sparse Row) form.
//!
//! This is the classic three-phase sparse *direct* method:
//!
//!   1. **transpose** — convert the CSR lower triangle to CSC so the
//!      left-looking recurrence has column access,
//!   2. **symbolic factorization** — compute the fill pattern (column
//!      structure) of `L` from the pattern of `A` by the left-looking closure
//!      `Struct(L[:,j]) = {j} ∪ Struct(A[:,j]) ∪ ⋃_{k<j, L[j,k]≠0} Struct(L[:,k])`,
//!   3. **numerical left-looking factorization** — restrict the standard
//!      Cholesky recurrence to that pattern.
//!
//! The result `L` is returned in CSR form (lower triangle only).
//!
//! ## CSR layout
//!
//! `row_ptr` has length `n+1`; the non-zeros of row `i` live in
//! `values[row_ptr[i] .. row_ptr[i+1]]` with columns
//! `col_idx[row_ptr[i] .. row_ptr[i+1]]`.  `A` is stored as a **full
//! symmetric** matrix (both triangles plus the diagonal, each non-zero once);
//! only the lower triangle (`i ≥ j`) is read by the algorithm.  The diagonal
//! must be present (a zero diagonal would make the matrix non
//! positive-definite and is rejected by the `assert!`).

/// Result of the sparse Cholesky factorization: the lower-triangular factor `L`
/// in CSR form.
pub struct SparseCholesky {
    pub l_values: Vec<f64>,
    pub l_col_idx: Vec<usize>,
    pub l_row_ptr: Vec<usize>,
}

/// Transpose a CSR matrix into CSC form (rows are emitted in increasing order
/// per column, so each column is sorted ascending by row).
///
/// Returns `(csc_values, csc_row_idx, csc_col_ptr)`.
fn csr_to_csc(
    values: &[f64],
    col_idx: &[usize],
    row_ptr: &[usize],
    n: usize,
) -> (Vec<f64>, Vec<usize>, Vec<usize>) {
    let nnz = values.len();

    // Count non-zeros per column.
    let mut count = vec![0usize; n];
    let mut p: usize = 0;
    while p < nnz {
        let col = col_idx[p];
        let t = count[col] + 1;
        count[col] = t;
        p += 1;
    }

    // Build column pointers.
    let mut col_ptr = vec![0usize; n + 1];
    let mut col: usize = 0;
    while col < n {
        let s = col_ptr[col] + count[col];
        col_ptr[col + 1] = s;
        col += 1;
    }

    // Fill the CSC arrays.
    let mut csc_values = vec![0.0; nnz];
    let mut csc_row_idx = vec![0usize; nnz];
    let mut next = vec![0usize; n];

    let mut row: usize = 0;
    while row < n {
        let mut p = row_ptr[row];
        while p < row_ptr[row + 1] {
            let col = col_idx[p];
            let slot = col_ptr[col] + next[col];
            csc_values[slot] = values[p];
            csc_row_idx[slot] = row;
            let t = next[col] + 1;
            next[col] = t;
            p += 1;
        }
        row += 1;
    }

    (csc_values, csc_row_idx, col_ptr)
}

/// Symbolic factorization: compute the sparsity pattern (column structure) of
/// the Cholesky factor `L`.
///
/// Returns `(l_row_idx, l_col_ptr)` in CSC form (rows within a column are not
/// sorted; the numerical phase is order-independent and `csr_decode` in the
/// spec is order-independent too).
fn symbolic_fill(
    a_row_idx: &[usize],
    a_col_ptr: &[usize],
    n: usize,
) -> (Vec<usize>, Vec<usize>) {
    let mut l_col_ptr = vec![0usize; n + 1];
    let mut l_row_idx: Vec<usize> = Vec::new();
    // Stamp array: `pos[row]` holds the column index `j` that most recently
    // added `row` to its pattern, or `n` (a sentinel larger than every column
    // index) if no such column has.  The stamp `j` is used to de-duplicate
    // within column `j`; because the stamp is the column index (never a global
    // position), it can never collide with the sentinel.
    let mut pos = vec![n; n];

    let mut j: usize = 0;
    while j < n {
        // Diagonal (j, j) is always present.
        l_row_idx.push(j);
        pos[j] = j;

        // Lower part of column j of A: rows i > j.
        let mut p = a_col_ptr[j];
        while p < a_col_ptr[j + 1] {
            let i = a_row_idx[p];
            if i > j {
                if pos[i] != j {
                    l_row_idx.push(i);
                    pos[i] = j;
                }
            }
            p += 1;
        }

        // Fill-in: for every column k < j that has a non-zero at row j
        // (i.e. L[j,k] != 0), union column k's pattern (rows > j) into j's.
        let mut k: usize = 0;
        while k < j {
            // Test whether row j is present in column k (linear scan).
            let mut q = l_col_ptr[k];
            let mut qj = l_col_ptr[k + 1];
            while q < l_col_ptr[k + 1] {
                if l_row_idx[q] == j {
                    qj = q;
                }
                q += 1;
            }
            if qj < l_col_ptr[k + 1] {
                let mut q2 = l_col_ptr[k];
                while q2 < l_col_ptr[k + 1] {
                    let i = l_row_idx[q2];
                    if i > j {
                        if pos[i] != j {
                            l_row_idx.push(i);
                            pos[i] = j;
                        }
                    }
                    q2 += 1;
                }
            }
            k += 1;
        }

        // Finalize column j (no `pos` reset is needed: column j+1 uses stamp
        // j+1, which differs from every earlier stamp).
        let cur = l_row_idx.len();
        l_col_ptr[j + 1] = cur;

        j += 1;
    }

    (l_row_idx, l_col_ptr)
}

/// Numerical left-looking Cholesky, restricted to the symbolic pattern.
///
/// Returns the CSC values of `L` (parallel to `l_row_idx`).
///
/// Recurrence for column j (left-looking):
///
/// ```text
///   x          = A[:, j]
///   x[j:n]    -= L[j, k] · L[j:n, k]     for every k < j with L[j, k] != 0
///   L[j, j]    = sqrt(x[j])
///   L[j+1:n,j] = x[j+1:n] / L[j, j]
/// ```
fn numeric_factor(
    a_values: &[f64],
    a_row_idx: &[usize],
    a_col_ptr: &[usize],
    l_row_idx: &[usize],
    l_col_ptr: &[usize],
    n: usize,
) -> Vec<f64> {
    let nnz_l = l_row_idx.len();
    let mut l_values = vec![0.0; nnz_l];
    let mut x = vec![0.0; n];

    let mut j: usize = 0;
    while j < n {
        // 1. Zero the work vector over the pattern of column j.
        let mut q = l_col_ptr[j];
        while q < l_col_ptr[j + 1] {
            let i = l_row_idx[q];
            x[i] = 0.0;
            q += 1;
        }

        // 2. Load column j of A (lower part) into x.
        let mut p = a_col_ptr[j];
        while p < a_col_ptr[j + 1] {
            let i = a_row_idx[p];
            if i >= j {
                x[i] = a_values[p];
            }
            p += 1;
        }

        // 3. Left-looking update from columns k < j with L[j, k] != 0.
        let mut k: usize = 0;
        while k < j {
            // Locate row j within column k (sentinel = end of column k).
            let mut q = l_col_ptr[k];
            let mut qj = l_col_ptr[k + 1];
            while q < l_col_ptr[k + 1] {
                if l_row_idx[q] == j {
                    qj = q;
                }
                q += 1;
            }
            if qj < l_col_ptr[k + 1] {
                let ljk = l_values[qj];
                let mut q2 = l_col_ptr[k];
                while q2 < l_col_ptr[k + 1] {
                    let i = l_row_idx[q2];
                    if i >= j {
                        let prod = ljk * l_values[q2];
                        let new_val = x[i] - prod;
                        x[i] = new_val;
                    }
                    q2 += 1;
                }
            }
            k += 1;
        }

        // 4. Diagonal: L[j, j] = sqrt(x[j]) (rejects non-positive pivots).
        let diag_val = x[j];
        assert!(diag_val > 0.0, "matrix is not positive-definite");
        let diag = diag_val.sqrt();

        // 5. Store column j: diagonal + normalized off-diagonal entries.
        let mut q = l_col_ptr[j];
        while q < l_col_ptr[j + 1] {
            let i = l_row_idx[q];
            if i == j {
                l_values[q] = diag;
            } else {
                let new_val = x[i] / diag;
                l_values[q] = new_val;
            }
            q += 1;
        }

        j += 1;
    }

    l_values
}

/// Convert a CSC matrix back into CSR form.
///
/// Returns `(csr_values, csr_col_idx, csr_row_ptr)`.
fn csc_to_csr(
    csc_values: &[f64],
    csc_row_idx: &[usize],
    csc_col_ptr: &[usize],
    n: usize,
) -> (Vec<f64>, Vec<usize>, Vec<usize>) {
    let nnz = csc_values.len();

    // Count non-zeros per row.
    let mut count = vec![0usize; n];
    let mut p: usize = 0;
    while p < nnz {
        let row = csc_row_idx[p];
        let t = count[row] + 1;
        count[row] = t;
        p += 1;
    }

    // Build row pointers.
    let mut row_ptr = vec![0usize; n + 1];
    let mut row: usize = 0;
    while row < n {
        let s = row_ptr[row] + count[row];
        row_ptr[row + 1] = s;
        row += 1;
    }

    // Fill the CSR arrays (columns come out ascending per row).
    let mut csr_values = vec![0.0; nnz];
    let mut csr_col_idx = vec![0usize; nnz];
    let mut next = vec![0usize; n];

    let mut col: usize = 0;
    while col < n {
        let mut p = csc_col_ptr[col];
        while p < csc_col_ptr[col + 1] {
            let row = csc_row_idx[p];
            let slot = row_ptr[row] + next[row];
            csr_values[slot] = csc_values[p];
            csr_col_idx[slot] = col;
            let t = next[row] + 1;
            next[row] = t;
            p += 1;
        }
        col += 1;
    }

    (csr_values, csr_col_idx, row_ptr)
}

/// Compute the sparse Cholesky factorization of a symmetric positive-definite
/// n×n matrix `A` given in CSR form (full symmetric matrix; only the lower
/// triangle is read).
///
/// Returns `SparseCholesky { l_values, l_col_idx, l_row_ptr }` such that
/// `csr_decode(L) · csr_decode(L)ᵀ = csr_decode(A)` with `L` lower-triangular
/// and having a positive diagonal.
///
/// # Panics
///
/// Panics if a diagonal pivot is not strictly positive, i.e. the matrix is not
/// positive-definite.
pub fn sparse_cholesky(
    a_values: &[f64],
    a_col_idx: &[usize],
    a_row_ptr: &[usize],
    n: usize,
) -> SparseCholesky {
    if n == 0 {
        let mut row_ptr: Vec<usize> = Vec::new();
        row_ptr.push(0);
        return SparseCholesky {
            l_values: Vec::new(),
            l_col_idx: Vec::new(),
            l_row_ptr: row_ptr,
        };
    }

    // 1. CSR -> CSC for column access.
    let (csc_values, csc_row_idx, csc_col_ptr) =
        csr_to_csc(a_values, a_col_idx, a_row_ptr, n);

    // 2. Symbolic fill pattern.
    let (pat_row_idx, pat_col_ptr) = symbolic_fill(&csc_row_idx, &csc_col_ptr, n);

    // 3. Numerical left-looking factorization.
    let l_csc_values = numeric_factor(
        &csc_values,
        &csc_row_idx,
        &csc_col_ptr,
        &pat_row_idx,
        &pat_col_ptr,
        n,
    );

    // 4. CSC -> CSR for the output.
    let (l_values, l_col_idx, l_row_ptr) =
        csc_to_csr(&l_csc_values, &pat_row_idx, &pat_col_ptr, n);

    SparseCholesky {
        l_values,
        l_col_idx,
        l_row_ptr,
    }
}
