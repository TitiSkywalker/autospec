//! Lindström–Gessel–Viennot (LGV) lemma: count non-intersecting paths in a
//! DAG via the determinant of a path-count matrix.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Exact determinant of an arbitrary integer matrix via Bareiss fraction-free
/// Gaussian elimination *with* partial pivoting.  Row swaps are counted (a
/// plain counter, since a self-referential `sign = -sign` toggle breaks
/// AENEAS's loop-fixed-point analysis); the parity is applied at the end.
fn det_int(n: usize, a: &[i64]) -> i64 {
    if n == 0 {
        return 1;
    }
    let mut m: Vec<i64> = a.to_vec();
    let mut swaps: usize = 0;
    let mut prev: i64 = 1;
    let mut k: usize = 0;
    while k + 1 < n {
        // Find a non-zero pivot in column `k`: the smallest row index `>= k`
        // with a non-zero entry (a sentinel `n` means none).
        let mut r: usize = n;
        let mut i: usize = k;
        while i < n {
            let mik = m[idx(i, n, k)];
            if mik != 0 {
                if i < r {
                    r = i;
                }
            }
            i += 1;
        }
        if r == n {
            return 0;
        }
        // Swap rows `r` and `k` (each swap flips the determinant's sign).
        if r != k {
            let mut j: usize = 0;
            while j < n {
                let a_rj = m[idx(r, n, j)];
                let a_kj = m[idx(k, n, j)];
                m[idx(r, n, j)] = a_kj;
                m[idx(k, n, j)] = a_rj;
                j += 1;
            }
            swaps += 1;
        }
        let pivot = m[idx(k, n, k)];
        // Bareiss update of the trailing submatrix (fraction-free).
        let mut i2: usize = k + 1;
        while i2 < n {
            let a_ik = m[idx(i2, n, k)];
            let mut j: usize = k + 1;
            while j < n {
                let a_ij = m[idx(i2, n, j)];
                let a_kj = m[idx(k, n, j)];
                let num = a_ij * pivot - a_ik * a_kj;
                let q = num / prev;
                m[idx(i2, n, j)] = q;
                j += 1;
            }
            i2 += 1;
        }
        prev = pivot;
        k += 1;
    }
    let last = m[idx(n - 1, n, n - 1)];
    let parity = swaps % 2;
    if parity == 0 {
        last
    } else {
        0 - last
    }
}

/// LGV lemma on a DAG: `det` of the path-count matrix `e(A_i, B_j)`.
///
/// The DAG is given by a flat, row-major, `n × n` 0/1 adjacency matrix `adj`
/// whose edges only go *forward* (`adj[u,v] = 1 ⟹ u < v`), so the vertices are
/// already topologically ordered.  `src` and `snk` are length-`k` slices of
/// distinct vertices; `src[i]` is the `i`-th source and `snk[j]` the `j`-th
/// sink (with `src[i] ≠ snk[j]` for all `i, j`).
///
/// Returns the determinant of the `k × k` matrix `M` with
/// `M[i,j] = e(src[i], snk[j])` = number of (unweighted) paths from source `i`
/// to sink `j`.  By the LGV lemma this determinant equals the signed sum over
/// vertex-disjoint path families.
pub fn graph_lgv_lemma(n: usize, adj: &[i64], k: usize, src: &[usize], snk: &[usize]) -> i64 {
    let mut e: Vec<i64> = vec![0i64; k * k];

    let mut s: usize = 0;
    while s < k {
        // Path-count DP from `src[s]` over the forward DAG:
        //   dp[v] = (v == src[s] ? 1 : 0) + Σ_{u < v} dp[u] * adj[u][v]
        let mut dp: Vec<i64> = vec![0i64; n];
        let mut v: usize = 0;
        while v < n {
            let mut sum: i64 = 0;
            if v == src[s] {
                sum = 1;
            }
            let mut u: usize = 0;
            while u < v {
                let dpu = dp[u];
                let auv = adj[idx(u, n, v)];
                let inc = dpu * auv;
                let new_sum = sum + inc;
                sum = new_sum;
                u += 1;
            }
            dp[v] = sum;
            v += 1;
        }
        let mut t: usize = 0;
        while t < k {
            let dst = snk[t];
            let val = dp[dst];
            e[idx(s, k, t)] = val;
            t += 1;
        }
        s += 1;
    }

    det_int(k, &e)
}
