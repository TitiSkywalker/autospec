//! Held–Karp dynamic program for the shortest Hamiltonian cycle (TSP).
//!
//! The input is a **complete** directed graph on the vertices `0 .. n`, given as
//! a flat row-major `n × n` distance matrix `dist`: `dist[v * n + u]` is the
//! distance from `v` to `u`.  Because the graph is complete, every ordering of
//! the vertices is a legitimate tour, so the shortest Hamiltonian-cycle cost is
//! always defined.

/// Shortest Hamiltonian-cycle cost through the vertices `0 .. n`.
///
/// `dp[mask * n + v]` is the length of the shortest path that starts at vertex
/// `0`, visits **exactly** the vertex set `mask` (so `0 ∈ mask`), and ends at
/// `v`.  Masks are processed in increasing order, so by the time `mask` is used
/// every sub-mask `mask \ {u}` has already been finalised.  The answer closes
/// the cycle by adding the edge `v → 0`.
///
/// Unreachable states keep the sentinel value `total + 1`, where `total` is the
/// sum of every matrix entry: a simple path uses each entry at most once, so no
/// realisable path length can reach the sentinel.  The sentinel is therefore
/// strictly larger than every reachable state, and it is never used in a `min`.
pub fn held_karp(n: usize, dist: &[usize]) -> usize {
    if n == 0 {
        return 0;
    }
    // Sentinel = (sum of all n×n matrix entries) + 1.
    let mut total: usize = 0;
    let mut a = 0;
    while a < n {
        let mut b = 0;
        while b < n {
            let w = dist[a * n + b];
            total += w;
            b += 1;
        }
        a += 1;
    }
    let sentinel = total + 1;

    let nstates = 1usize << n;
    let full = nstates - 1;
    let size = nstates * n;
    let mut dp = vec![sentinel; size];
    // dp[mask = {0}][v = 0] = 0
    dp[n] = 0;

    let mut mask = 1;
    while mask <= full {
        let mut v = 0;
        while v < n {
            let cur = dp[mask * n + v];
            if cur != sentinel {
                let mut u = 0;
                while u < n {
                    let bit = 1usize << u;
                    let seen = mask & bit;
                    if seen == 0 {
                        let nmask = mask | bit;
                        let cand = cur + dist[v * n + u];
                        let idx = nmask * n + u;
                        let old = dp[idx];
                        if cand < old {
                            dp[idx] = cand;
                        }
                    }
                    u += 1;
                }
            }
            v += 1;
        }
        mask += 1;
    }

    // Close the cycle back to vertex 0.
    let mut best = sentinel;
    let mut v = 0;
    while v < n {
        let cur = dp[full * n + v];
        if cur != sentinel {
            let cand = cur + dist[v * n];
            if cand < best {
                best = cand;
            }
        }
        v += 1;
    }
    best
}
