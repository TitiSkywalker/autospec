//! Chromatic number of a graph — subset DP over independent sets.
//!
//! The graph on `n` vertices is given as a flat row-major `n × n` adjacency
//! matrix `adj`: `adj[u * n + v] != 0` means `u` and `v` are adjacent (for the
//! diagonal this reads `adj[u * n + u] != 0`, i.e. a self-loop, which the
//! caller must avoid — the algorithm assumes an irreflexive graph).
//!
//! `dp[mask]` is the chromatic number of the subgraph induced by the vertex
//! set `mask`:
//!
//! * `dp[0] = 0`;
//! * otherwise, `dp[mask] = min over independent S ⊆ mask, S ≠ ∅ of
//!   1 + dp[mask ^ S]` — the colour class of some vertex is an independent set
//!   `S`, and removing it leaves a strictly smaller induced subgraph.
//!
//! The answer is `dp[2^n - 1]`.
//!
//! Requires `n <= 20` (the DP enumerates `2^n` masks and `2^n` frontiers).

/// Flat row-major index of entry `(row, col)` in an `n × n` matrix.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// `1` if the vertex subset `mask` is an independent set, else `0`.
///
/// `mask` is independent when no two of its vertices are joined by an edge;
/// the adjacency test is exactly `adj[u * n + v] != 0`, over *all* ordered
/// pairs `(u, v)` contained in `mask` (so a non-zero diagonal blocks every
/// non-empty independent set, matching the self-loop reading above).
fn mask_independent(n: usize, adj: &[usize], mask: usize) -> usize {
    let mut bad = 0usize;
    let mut u = 0;
    while u < n {
        let mut v = 0;
        while v < n {
            let bu = (mask >> u) & 1usize;
            let bv = (mask >> v) & 1usize;
            let both = bu * bv;
            let a = adj[idx(u, n, v)];
            let isedge = if a != 0 { 1usize } else { 0usize };
            let contrib = both * isedge;
            let bad2 = bad + contrib;
            bad = bad2;
            v += 1;
        }
        u += 1;
    }
    if bad != 0 { 0 } else { 1 }
}

/// Minimum number of colours needed to colour the graph's `n` vertices so
/// that adjacent vertices get different colours (the chromatic number).
pub fn chromatic_number(n: usize, adj: &[usize]) -> usize {
    let total = 1usize << n;
    let mut dp = vec![0usize; total];
    let mut ind = vec![0usize; total];

    let mut m = 0;
    while m < total {
        let v = mask_independent(n, adj, m);
        ind[m] = v;
        m += 1;
    }

    let mut mask = 1;
    while mask < total {
        // A single vertex is always an independent set, so `n` colours always
        // suffice: this is the running upper bound.
        dp[mask] = n;
        let mut sub = mask;
        while sub != 0 {
            let isind = ind[sub];
            if isind != 0 {
                let rest = mask ^ sub;
                let d = dp[rest];
                let cand = d + 1;
                let cur = dp[mask];
                if cand < cur {
                    dp[mask] = cand;
                }
            }
            let nsub = (sub - 1) & mask;
            sub = nsub;
        }
        mask += 1;
    }

    let last = total - 1;
    dp[last]
}
