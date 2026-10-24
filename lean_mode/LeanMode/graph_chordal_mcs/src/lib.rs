//! Maximum Cardinality Search (MCS) ordering of an undirected graph, read off
//! a flat row-major adjacency matrix.
//!
//! The MCS ordering is the standard tool behind the Fulkerson–Gross / Rose–
//! Tarjan–Lueker characterisation of chordal graphs: an MCS ordering is a
//! perfect elimination ordering exactly when the graph is chordal.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Maximum Cardinality Search (MCS) on an undirected graph.
///
/// `adj` is a flat, row-major `n × n` symmetric `0/1` adjacency matrix:
/// `adj[u * n + v] == 1` iff `{u, v}` is an edge.
///
/// Returns the MCS ordering `pi` of length `n`.  Concretely, the vertex chosen
/// first (an unnumbered vertex with the most already-numbered neighbours,
/// ties broken by the smallest vertex index) receives number `n`, the next
/// chosen receives number `n - 1`, ..., the last chosen receives number `1`;
/// `pi[k]` is the vertex numbered `k + 1`.  (This is the standard MCS
/// numbering, from `n` down to `1`, so that `pi` is the perfect-elimination
/// orientation of the MCS order; it is a perfect elimination ordering exactly
/// when the graph is chordal.)
pub fn chordal_mcs(n: usize, adj: &[usize]) -> Vec<usize> {
    let mut pi: Vec<usize> = vec![0usize; n];
    let mut weight: Vec<usize> = vec![0usize; n];
    let mut numbered: Vec<usize> = vec![0usize; n];

    // `slot` counts down from `n`: the vertex chosen at step `k` (0-based) is
    // written to `pi[n - 1 - k]`.
    let mut slot: usize = n;
    while slot > 0 {
        slot -= 1;

        // Select an unnumbered vertex of maximum weight; ties -> smallest
        // index.  A downward scan updating on `>=` keeps the smallest
        // maximiser (and always finds one while `slot` vertices remain).
        let mut best: usize = 0;
        let mut bestw: usize = 0;
        let mut i: usize = n;
        while i > 0 {
            i -= 1;
            let nb = numbered[i];
            if nb == 0 {
                let w = weight[i];
                if w >= bestw {
                    best = i;
                    bestw = w;
                }
            }
        }

        pi[slot] = best;
        numbered[best] = 1;

        // Bump the weight of every neighbour of the newly numbered vertex.
        let mut v: usize = 0;
        while v < n {
            let a = adj[idx(best, n, v)];
            if a != 0 {
                let w = weight[v];
                let nw = w + 1;
                weight[v] = nw;
            }
            v += 1;
        }
    }

    pi
}
