//! Tests for the symbolic factorization / elimination tree.
//!
//! The expected fill patterns were computed independently from the *definition*
//! of the fill (eliminate the vertices in increasing order, adding a clique on
//! the remaining neighbours of each eliminated vertex) — equivalently from the
//! Rose–Tarjan–Lueker fill path theorem — not from the left-looking recurrence
//! implemented here.

/// Dense 0/1 row-major view of a CSC pattern (`col_ptr`/`row_idx`).
fn dense_l(row_idx: &[usize], col_ptr: &[usize], n: usize) -> Vec<usize> {
    let mut d = vec![0usize; n * n];
    let mut j: usize = 0;
    while j < n {
        let mut q = col_ptr[j];
        while q < col_ptr[j + 1] {
            let i = row_idx[q];
            d[i * n + j] = 1;
            q += 1;
        }
        j += 1;
    }
    d
}

/// Element-wise equality.
fn veq(a: &[usize], b: &[usize]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut same = true;
    let mut i: usize = 0;
    while i < a.len() {
        if a[i] != b[i] {
            same = false;
        }
        i += 1;
    }
    same
}

/// The column pointers are non-decreasing.
fn monotone(ptr: &[usize]) -> bool {
    let mut ok = true;
    let mut i: usize = 0;
    while i + 1 < ptr.len() {
        if ptr[i] > ptr[i + 1] {
            ok = false;
        }
        i += 1;
    }
    ok
}

/// The pattern of `L` is lower triangular (diagonal included).
fn lower_ok(dl: &[usize], n: usize) -> bool {
    let mut ok = true;
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            if dl[i * n + j] == 1 {
                if i < j {
                    ok = false;
                }
            }
            j += 1;
        }
        i += 1;
    }
    ok
}

/// Run the symbolic factorization and compare the whole output with the
/// expected pattern of `L` (dense row-major, 0/1) and elimination tree.
fn check_symbolic(
    col_idx: &[usize],
    row_ptr: &[usize],
    n: usize,
    expected_l: &[usize],
    expected_parent: &[usize],
) {
    let res = matrix_sparse_cholesky_symbolic::symbolic_factorization(col_idx, row_ptr, n);

    // Shape of the output.
    assert_eq!(res.l_col_ptr.len(), n + 1);
    assert_eq!(res.parent.len(), n);
    assert!(monotone(&res.l_col_ptr));

    // The pattern of L and the elimination tree.
    let dl = dense_l(&res.l_row_idx, &res.l_col_ptr, n);
    assert!(veq(&dl, expected_l));
    assert!(lower_ok(&dl, n));
    assert!(veq(&res.parent, expected_parent));
}

/// A single vertex: no fill, the tree is a bare root.
#[test]
fn test_symbolic_single() {
    let col_idx = vec![0usize];
    let row_ptr = vec![0usize, 1];
    let l = vec![1usize];
    let parent = vec![1usize];
    check_symbolic(&col_idx, &row_ptr, 1, &l, &parent);
}

/// The empty matrix.
#[test]
fn test_symbolic_empty() {
    let col_idx: Vec<usize> = Vec::new();
    let row_ptr = vec![0usize];
    let l: Vec<usize> = Vec::new();
    let parent: Vec<usize> = Vec::new();
    check_symbolic(&col_idx, &row_ptr, 0, &l, &parent);
}

/// A tridiagonal (path) pattern creates no fill.
#[test]
fn test_symbolic_tridiagonal() {
    let col_idx = vec![0usize, 1, 0, 1, 2, 1, 2, 3, 2, 3];
    let row_ptr = vec![0usize, 2, 5, 8, 10];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0,
        1, 1, 0, 0,
        0, 1, 1, 0,
        0, 0, 1, 1,
    ];
    let parent = vec![1usize, 2, 3, 4];
    check_symbolic(&col_idx, &row_ptr, 4, &l, &parent);
}

/// An arrow (star centred at 0): eliminating vertex 0 makes its three
/// neighbours a clique — the pattern of L is the full lower triangle.
#[test]
fn test_symbolic_arrow() {
    let col_idx = vec![0usize, 1, 2, 3, 0, 1, 0, 2, 0, 3];
    let row_ptr = vec![0usize, 4, 6, 8, 10];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0,
        1, 1, 0, 0,
        1, 1, 1, 0,
        1, 1, 1, 1,
    ];
    let parent = vec![1usize, 2, 3, 4];
    check_symbolic(&col_idx, &row_ptr, 4, &l, &parent);
}

/// A 4-cycle: eliminating 0 fills in the single entry (3, 1).
#[test]
fn test_symbolic_cycle4() {
    let col_idx = vec![0usize, 1, 3, 0, 1, 2, 1, 2, 3, 0, 2, 3];
    let row_ptr = vec![0usize, 3, 6, 9, 12];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0,
        1, 1, 0, 0,
        0, 1, 1, 0,
        1, 1, 1, 1,
    ];
    let parent = vec![1usize, 2, 3, 4];
    check_symbolic(&col_idx, &row_ptr, 4, &l, &parent);
}

/// The complete graph K4: no fill at all.
#[test]
fn test_symbolic_complete4() {
    let col_idx = vec![0usize, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3];
    let row_ptr = vec![0usize, 4, 8, 12, 16];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0,
        1, 1, 0, 0,
        1, 1, 1, 0,
        1, 1, 1, 1,
    ];
    let parent = vec![1usize, 2, 3, 4];
    check_symbolic(&col_idx, &row_ptr, 4, &l, &parent);
}

/// A path with one long chord (0—4): the fill propagates along the path.
#[test]
fn test_symbolic_path_with_chord() {
    let col_idx = vec![0usize, 1, 4, 0, 1, 2, 1, 2, 3, 2, 3, 4, 0, 3, 4];
    let row_ptr = vec![0usize, 3, 6, 9, 12, 15];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0, 0,
        1, 1, 0, 0, 0,
        0, 1, 1, 0, 0,
        0, 0, 1, 1, 0,
        1, 1, 1, 1, 1,
    ];
    let parent = vec![1usize, 2, 3, 4, 5];
    check_symbolic(&col_idx, &row_ptr, 5, &l, &parent);
}

/// Two disconnected triangles: two independent roots.
#[test]
fn test_symbolic_two_components() {
    let col_idx = vec![
        0usize, 1, 2, 0, 1, 2, 0, 1, 2, 3, 4, 5, 3, 4, 5, 3, 4, 5,
    ];
    let row_ptr = vec![0usize, 3, 6, 9, 12, 15, 18];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0, 0, 0,
        1, 1, 0, 0, 0, 0,
        1, 1, 1, 0, 0, 0,
        0, 0, 0, 1, 0, 0,
        0, 0, 0, 1, 1, 0,
        0, 0, 0, 1, 1, 1,
    ];
    let parent = vec![1usize, 2, 6, 4, 5, 6];
    check_symbolic(&col_idx, &row_ptr, 6, &l, &parent);
}

/// One edge plus two isolated vertices: the isolated columns have no entry
/// below the diagonal, so their parent is the sentinel `n`.
#[test]
fn test_symbolic_isolated() {
    let col_idx = vec![0usize, 1, 0, 1, 2, 3];
    let row_ptr = vec![0usize, 2, 4, 5, 6];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0,
        1, 1, 0, 0,
        0, 0, 1, 0,
        0, 0, 0, 1,
    ];
    let parent = vec![1usize, 4, 4, 4];
    check_symbolic(&col_idx, &row_ptr, 4, &l, &parent);
}

/// A 3×3 grid (9 vertices): several layers of fill.
#[test]
fn test_symbolic_grid3x3() {
    let col_idx = vec![
        0usize, 1, 3, 0, 1, 2, 4, 1, 2, 5, 0, 3, 4, 6, 1, 3, 4, 5, 7, 2, 4, 5, 8, 3, 6, 7, 4, 6, 7,
        8, 5, 7, 8,
    ];
    let row_ptr = vec![0usize, 3, 7, 10, 14, 19, 23, 26, 30, 33];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0, 0, 0, 0, 0, 0,
        1, 1, 0, 0, 0, 0, 0, 0, 0,
        0, 1, 1, 0, 0, 0, 0, 0, 0,
        1, 1, 1, 1, 0, 0, 0, 0, 0,
        0, 1, 1, 1, 1, 0, 0, 0, 0,
        0, 0, 1, 1, 1, 1, 0, 0, 0,
        0, 0, 0, 1, 1, 1, 1, 0, 0,
        0, 0, 0, 0, 1, 1, 1, 1, 0,
        0, 0, 0, 0, 0, 1, 1, 1, 1,
    ];
    let parent = vec![1usize, 2, 3, 4, 5, 6, 7, 8, 9];
    check_symbolic(&col_idx, &row_ptr, 9, &l, &parent);
}

/// A path on six vertices with two extra edges — a column whose fill entries
/// come purely from transitive closure ((5, 2) and (5, 3)).
#[test]
fn test_symbolic_transitive_fill() {
    let col_idx = vec![0usize, 1, 5, 0, 1, 2, 5, 1, 2, 3, 2, 3, 4, 3, 4, 5, 0, 1, 4, 5];
    let row_ptr = vec![0usize, 3, 7, 10, 13, 16, 20];
    #[rustfmt::skip]
    let l = vec![
        1usize, 0, 0, 0, 0, 0,
        1, 1, 0, 0, 0, 0,
        0, 1, 1, 0, 0, 0,
        0, 0, 1, 1, 0, 0,
        0, 0, 0, 1, 1, 0,
        1, 1, 1, 1, 1, 1,
    ];
    let parent = vec![1usize, 2, 3, 4, 5, 6];
    check_symbolic(&col_idx, &row_ptr, 6, &l, &parent);
}
