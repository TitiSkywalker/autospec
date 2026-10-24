//! Number of spanning trees of an undirected graph via the matrix-tree theorem
//! (Kirchhoff's theorem): determinant of a Laplace (Kirchhoff) cofactor.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Exact determinant of a positive-semidefinite matrix (e.g. a Laplace
/// cofactor) via Bareiss fraction-free Gaussian elimination *without*
/// pivoting.  For such a matrix the leading principal minors are all
/// non-negative, so a zero pivot exactly detects singularity (determinant 0).
fn det_int(n: usize, a: &[i64]) -> i64 {
    if n == 0 {
        return 1;
    }
    let mut m: Vec<i64> = a.to_vec();
    let mut prev: i64 = 1;
    let mut k: usize = 0;
    while k + 1 < n {
        let pivot = m[idx(k, n, k)];
        if pivot == 0 {
            return 0;
        }
        let mut i: usize = k + 1;
        while i < n {
            let mut j: usize = k + 1;
            while j < n {
                let a_ij = m[idx(i, n, j)];
                let a_ik = m[idx(i, n, k)];
                let a_kj = m[idx(k, n, j)];
                let num = a_ij * pivot - a_ik * a_kj;
                let q = num / prev;
                m[idx(i, n, j)] = q;
                j += 1;
            }
            i += 1;
        }
        prev = pivot;
        k += 1;
    }
    let last = m[idx(n - 1, n, n - 1)];
    last
}

/// Count the spanning trees of an undirected graph via the matrix-tree
/// theorem.
///
/// `adj` is a flat, row-major, symmetric `n × n` adjacency matrix (0/1, no
/// self-loops): `adj[idx(u, n, v)] == 1` when `u` and `v` are adjacent.  The
/// number of spanning trees equals any cofactor of the Laplace matrix
/// `L = D - A` (`D` = degree matrix), computed here by dropping the last row
/// and column and taking the exact determinant (Bareiss).
///
/// Returns the number of spanning trees (0 for a disconnected graph).
pub fn matrix_tree(n: usize, adj: &[i64]) -> i64 {
    // Degrees.
    let mut deg: Vec<i64> = vec![0i64; n];
    let mut i: usize = 0;
    while i < n {
        let mut d: i64 = 0;
        let mut j: usize = 0;
        while j < n {
            let a = adj[idx(i, n, j)];
            d = d + a;
            j += 1;
        }
        deg[i] = d;
        i += 1;
    }

    // Laplace matrix `L = D - A`.
    let mut lap: Vec<i64> = vec![0i64; n * n];
    let mut i2: usize = 0;
    while i2 < n {
        let mut j: usize = 0;
        while j < n {
            if i2 == j {
                lap[idx(i2, n, j)] = deg[i2];
            } else {
                let a = adj[idx(i2, n, j)];
                let neg = 0 - a;
                lap[idx(i2, n, j)] = neg;
            }
            j += 1;
        }
        i2 += 1;
    }

    if n <= 1 {
        return 1;
    }

    // Cofactor: drop the last row and column.
    let m = n - 1;
    let mut cof: Vec<i64> = vec![0i64; m * m];
    let mut i3: usize = 0;
    while i3 < m {
        let mut j: usize = 0;
        while j < m {
            let l = lap[idx(i3, n, j)];
            cof[idx(i3, m, j)] = l;
            j += 1;
        }
        i3 += 1;
    }

    det_int(m, &cof)
}
