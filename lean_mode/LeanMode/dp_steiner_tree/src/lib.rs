//! Minimum Steiner tree (Dreyfus–Wagner dynamic program).
//!
//! The input is an **undirected** weighted graph on `n` vertices, given as a
//! flat row-major `n × n` weight matrix `w`: `w[u * n + v] = w[v * n + u]` is
//! the weight of the edge `u–v`.  The graph is complete, so a tree connecting
//! any set of terminals always exists.
//!
//! `dp[mask * n + v]` is the minimum total weight of a tree that connects every
//! terminal whose index appears in `mask` and that contains the vertex `v`.
//! Every mask is finalised in two steps, in this order:
//!
//! 1. **merge** — split `mask` into two disjoint non-empty parts `sub` and
//!    `mask - sub` and join the two trees at their common root `v`;
//! 2. **relax** — let the root wander along shortest paths: `n` Bellman–Ford
//!    relaxation rounds over all edges.
//!
//! Masks are processed in increasing order, so both parts of a split are
//! already final when the split is used.  The answer is the best value over all
//! roots for the full mask.
//!
//! Unreachable states start at the sentinel `1 + Σ w`, and no realisable tree
//! can cost that much (a tree uses every matrix entry at most once), so the
//! sentinel never wins a `min`.  In a complete graph every state is reachable
//! anyway; the sentinel only initialises the table.

/// Minimum total weight of a tree connecting every terminal.
///
/// `w` is the flat row-major `n × n` weight matrix (symmetric), `terminals` the
/// list of vertex indices to connect.  Returns `0` when fewer than two
/// terminals have to be connected.
pub fn steiner_tree(n: usize, w: &[usize], terminals: &[usize]) -> usize {
    let k = terminals.len();
    if k < 2 {
        return 0;
    }
    // Sentinel = (sum of every matrix entry) + 1.
    let mut total: usize = 0;
    let mut a = 0;
    while a < n {
        let mut b = 0;
        while b < n {
            let x = w[a * n + b];
            total += x;
            b += 1;
        }
        a += 1;
    }
    let sentinel = total + 1;

    let nstates = 1usize << k;
    let full = nstates - 1;
    let mut dp = vec![sentinel; nstates * n];
    // dp[{i}][terminals[i]] = 0
    let mut i = 0;
    while i < k {
        let t = terminals[i];
        let idx = (1usize << i) * n + t;
        dp[idx] = 0;
        i += 1;
    }

    let mut mask = 1;
    while mask <= full {
        // ── merge: join two disjoint sub-trees at their common root v ───────
        let mut sub = mask;
        while sub > 0 {
            let rest = mask - sub;
            let mut v = 0;
            while v < n {
                let left = dp[sub * n + v];
                let right = dp[rest * n + v];
                let cand = left + right;
                let idx = mask * n + v;
                let old = dp[idx];
                if cand < old {
                    dp[idx] = cand;
                }
                v += 1;
            }
            sub = (sub - 1usize) & mask;
        }
        // ── relax: root may move along any edge, n Bellman–Ford rounds ─────
        let mut round = 0;
        while round < n {
            let mut u = 0;
            while u < n {
                let du = dp[mask * n + u];
                let mut v = 0;
                while v < n {
                    let cand = du + w[u * n + v];
                    let idx = mask * n + v;
                    let old = dp[idx];
                    if cand < old {
                        dp[idx] = cand;
                    }
                    v += 1;
                }
                u += 1;
            }
            round += 1;
        }
        mask += 1;
    }

    let mut best = sentinel;
    let mut v = 0;
    while v < n {
        let cur = dp[full * n + v];
        if cur < best {
            best = cur;
        }
        v += 1;
    }
    best
}
