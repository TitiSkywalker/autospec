//! *Symbolic* phase of the sparse Cholesky factorization, plus the
//! elimination tree.
//!
//! Given only the **sparsity pattern** of a symmetric n×n matrix `A` (in CSR
//! form), this computes
//!
//!   * the sparsity pattern of the Cholesky factor `L` — i.e. the pattern of
//!     `A` *plus the fill-in* created by the elimination, and
//!   * the **elimination tree** `parent`, where
//!     `parent[j] = min { i > j : L[i][j] ≠ 0 }` (and `n` when column `j` of
//!     `L` has no entry below the diagonal).
//!
//! No numerical values are involved: with exact arithmetic a fill entry may
//! still cancel numerically, but structurally it cannot be avoided.  The
//! *fill path theorem* of Rose–Tarjan–Lueker characterises the pattern of `L`
//! exactly: for `i > j` the entry `L[i][j]` is structurally non-zero iff
//! `A[i][j] ≠ 0` or there is a path `i = v₀, v₁, …, v_k = j` in the graph of
//! `A` whose *interior* vertices `v₁, …, v_{k-1}` are all smaller than `j`.
//! The formal statement of that equivalence is in `Spec.lean`.
//!
//! ## CSR layout
//!
//! `row_ptr` has length `n+1`; the non-zeros of row `i` live in
//! `col_idx[row_ptr[i] .. row_ptr[i+1]]`.  `A` is a **full symmetric** pattern
//! (each non-zero is stored in both triangles); only the lower triangle
//! (`i ≥ j`) is used by the algorithm.
//!
//! ## Algorithm
//!
//! The pattern of `L` is computed by the left-looking *symbolic* recurrence
//! (`Struct` = the set of structurally non-zero positions)
//!
//! ```text
//!   Struct(L[:,j]) = {j} ∪ {i > j : A[i][j] ≠ 0}
//!                    ∪ ⋃ { Struct(L[:,k]) ∩ {i > j} : k < j, L[j][k] ≠ 0 }
//! ```
//!
//! which is exactly the fill created by eliminating the vertices `0, 1, …, j-1`
//! in order.

/// Result of the symbolic factorization.
///
/// * `l_col_ptr` (`n+1` entries) / `l_row_idx` — the CSC pattern of `L`
///   (lower triangle + diagonal; the rows within a column are unordered),
/// * `parent` (`n` entries) — the elimination tree, `n` marking a root.
pub struct SymbolicFactorization {
    pub l_col_ptr: Vec<usize>,
    pub l_row_idx: Vec<usize>,
    pub parent: Vec<usize>,
}

/// Transpose the CSR pattern of `A` into CSC form, so that the columns needed
/// by the left-looking recurrence can be walked.
///
/// Returns `(csc_row_idx, csc_col_ptr)`; within a column, rows are emitted in
/// increasing order.
fn pattern_to_csc(
    col_idx: &[usize],
    row_ptr: &[usize],
    n: usize,
) -> (Vec<usize>, Vec<usize>) {
    let nnz = col_idx.len();

    let mut count = vec![0usize; n];
    let mut p: usize = 0;
    while p < nnz {
        let col = col_idx[p];
        let t = count[col] + 1;
        count[col] = t;
        p += 1;
    }

    let mut csc_col_ptr = vec![0usize; n + 1];
    let mut col: usize = 0;
    while col < n {
        let s = csc_col_ptr[col] + count[col];
        csc_col_ptr[col + 1] = s;
        col += 1;
    }

    let mut csc_row_idx = vec![0usize; nnz];
    let mut next = vec![0usize; n];

    let mut row: usize = 0;
    while row < n {
        let mut p2 = row_ptr[row];
        while p2 < row_ptr[row + 1] {
            let col = col_idx[p2];
            let slot = csc_col_ptr[col] + next[col];
            csc_row_idx[slot] = row;
            let t = next[col] + 1;
            next[col] = t;
            p2 += 1;
        }
        row += 1;
    }

    (csc_row_idx, csc_col_ptr)
}

/// Left-looking symbolic factorization: the CSC pattern of `L`.
///
/// Returns `(l_row_idx, l_col_ptr)`.
fn symbolic_fill(
    a_row_idx: &[usize],
    a_col_ptr: &[usize],
    n: usize,
) -> (Vec<usize>, Vec<usize>) {
    let mut l_col_ptr = vec![0usize; n + 1];
    let mut l_row_idx: Vec<usize> = Vec::new();
    // `pos[row]` holds the column that most recently inserted `row` into its
    // pattern, or the sentinel `n`.  The stamp is the *column index*, which can
    // never collide with the sentinel, so no reset pass between columns is
    // needed.
    let mut pos = vec![n; n];

    let mut j: usize = 0;
    while j < n {
        // The diagonal is always structurally non-zero.
        l_row_idx.push(j);
        pos[j] = j;

        // Lower part of column j of A (rows i > j).
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

        // Fill-in: for every k < j with L[j][k] != 0 (row j present in column
        // k), union column k's pattern below row j into column j.
        let mut k: usize = 0;
        while k < j {
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

        let cur = l_row_idx.len();
        l_col_ptr[j + 1] = cur;

        j += 1;
    }

    (l_row_idx, l_col_ptr)
}

/// The elimination tree of the pattern of `L`:
/// `parent[j] = min { i > j : L[i][j] ≠ 0 }`, or `n` if there is none.
fn elimination_tree(l_row_idx: &[usize], l_col_ptr: &[usize], n: usize) -> Vec<usize> {
    let mut parent = vec![n; n];

    let mut j: usize = 0;
    while j < n {
        let mut best = n;
        let mut q = l_col_ptr[j];
        while q < l_col_ptr[j + 1] {
            let i = l_row_idx[q];
            if i > j {
                if i < best {
                    best = i;
                }
            }
            q += 1;
        }
        parent[j] = best;
        j += 1;
    }

    parent
}

/// Symbolic factorization of the symmetric pattern of `A`
/// (`col_idx`/`row_ptr`: CSR, full symmetric, see the module documentation).
///
/// Returns the CSC pattern of `L` together with the elimination tree.  The
/// input pattern must be symmetric; only its lower triangle is read.
pub fn symbolic_factorization(
    col_idx: &[usize],
    row_ptr: &[usize],
    n: usize,
) -> SymbolicFactorization {
    let (csc_row_idx, csc_col_ptr) = pattern_to_csc(col_idx, row_ptr, n);
    let (l_row_idx, l_col_ptr) = symbolic_fill(&csc_row_idx, &csc_col_ptr, n);
    let parent = elimination_tree(&l_row_idx, &l_col_ptr, n);

    SymbolicFactorization {
        l_col_ptr: l_col_ptr,
        l_row_idx: l_row_idx,
        parent: parent,
    }
}
