//! `graph_weighted_matching` — maximum-weight matching in a general
//! (non-bipartite) undirected graph.
//!
//! # Input convention
//!
//! * `adj` is a flat, row-major `n × n` **0/1 adjacency matrix**
//!   (`adj[u * n + v] == 1` iff `u` and `v` are adjacent; symmetric, no
//!   self-loops) — exactly the convention of `graph_general_match`.
//! * `w` is a flat, row-major `n × n` weight matrix.  Only the entries on
//!   edges matter; `w[u * n + v]` is the weight of the edge `{u, v}` (it may be
//!   negative — such an edge is then simply never used).  `w` is assumed
//!   symmetric.
//!
//! The edge test in the implementation is the literal comparison
//! `adj[u * n + v] == 1`, and the specification's adjacency relation is the
//! *same* predicate, so the two describe the same graph.
//!
//! # Algorithm (shipped variant)
//!
//! A full primal-dual Edmonds weighted-blossom (Edmonds 1965) is the algorithm
//! this testcase was proposed around, but its correct form needs *nested*
//! blossom expansion together with vertex **and** odd-set dual variables; the
//! odd-set (blossom) duals are exactly what makes the strong-duality certificate
//! `weight = Σ y + Σ_B (|B|-1)/2 · z_B` true — the vertex-only dual is false
//! (the unit triangle has max matching weight `1` while the vertex-only dual
//! optimum is `3/2`).
//!
//! The shipped body is instead the exact **O(2^n) subset dynamic program** (the
//! documented fallback), which computes the same function and validates the
//! same specification.
//!
//! A full O(n^3) primal-dual Edmonds weighted blossom (the "relabeling" /
//! blossom-tree method, with dual labels `lab` on vertices and blossoms, an
//! outer/inner alternating forest, `slack`, `get_lca`, `add_blossom`,
//! `expand_blossom`, `set_match`, `augment`) *was* implemented here and does
//! compile and translate, but it failed the exhaustive brute-force differential
//! test (e.g. the 4-vertex graph with edges `0-2 (8), 0-3 (5), 1-3 (5),
//! 2-3 (10)` returns weight `10` where the optimum is `13`).  The defect is the
//! blossom-augmentation bookkeeping: `set_match`/`augment` must route the
//! augmenting path through the *member of the blossom that is the endpoint of
//! the tight edge*, which the canonical implementations do with an ordered
//! member list plus `get_pr`-style cyclic rotation; this was not reconstructed
//! correctly (and half-integral `δ` handling interacts with it).  Without a
//! reference implementation available, converging it to full correctness was
//! out of scope, so the verified subset DP is shipped.
//!
//! `dp[S]` is the maximum total weight of a matching that only uses vertices of
//! the set `S` (an `n`-bit mask):
//!
//! * `dp[0] = 0`;
//! * for `S ≠ ∅`, let `u` be the lowest-indexed vertex of `S`.  Then either `u`
//!   is unmatched (`dp[S \ {u}]`) or it is matched to some `v ∈ S \ {u}` with
//!   `adj[u, v] = 1` (`w[u, v] + dp[S \ {u, v}]`); take the maximum.
//!
//! Every matching is considered exactly once (its lowest vertex is decided
//! first), so `dp[2^n − 1]` is the maximum weight matching.  `choice[S]` records
//! the partner chosen for the lowest vertex of `S`, which rebuilds the matching.
//!
//! Requires `n ≤ 20` (the DP stores `2^n` entries).

/// Flat row-major index of entry `(row, col)` in an `n × n` matrix.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Maximum-weight matching of the graph `adj` with edge weights `w`.
///
/// Returns `(total, matched)` where `total` is the maximum total weight of a
/// matching and `matched` is a flat row-major `n × n` 0/1 matrix marking the
/// matched edges (`matched[u * n + v] == 1` iff `u` is matched to `v`).
pub fn max_weight_matching(n: usize, adj: &[i64], w: &[i64]) -> (i64, Vec<i64>) {
    let total = 1usize << n;
    // dp[S] = max weight matching on vertex set S (bitmask).
    let mut dp: Vec<i64> = vec![0i64; total];
    // choice[S] = the partner of the lowest vertex of S in an optimal matching
    // of S, or n if that vertex is left unmatched.
    let mut choice: Vec<usize> = vec![n; total];

    let mut s: usize = 1;
    while s < total {
        // Lowest-indexed vertex of s (s != 0 here, so the scan always sets u).
        let mut u: usize = n;
        let mut i: usize = 0;
        while i < n {
            let b = (s >> i) & 1usize;
            if b == 1usize {
                if i < u {
                    u = i;
                }
            }
            i += 1;
        }
        let sm1 = s - 1usize;
        let rest = s & sm1; // s without its lowest vertex u
        let mut best = dp[rest]; // u unmatched
        let mut v: usize = 0;
        while v < n {
            let bv = (s >> v) & 1usize;
            if bv == 1usize {
                if v != u {
                    let a = adj[idx(u, n, v)];
                    if a == 1i64 {
                        let bitv = 1usize << v;
                        let rest2 = rest ^ bitv; // s without u and without v
                        let d2 = dp[rest2];
                        let ww = w[idx(u, n, v)];
                        let cand = ww + d2;
                        if cand > best {
                            best = cand;
                            choice[s] = v;
                        }
                    }
                }
            }
            v += 1;
        }
        dp[s] = best;
        s += 1;
    }

    // Rebuild the matching by following `choice` from the full vertex set.
    let mut matched: Vec<i64> = vec![0i64; n * n];
    let mut s2: usize = total - 1usize;
    while s2 != 0 {
        let mut u: usize = n;
        let mut i: usize = 0;
        while i < n {
            let b = (s2 >> i) & 1usize;
            if b == 1usize {
                if i < u {
                    u = i;
                }
            }
            i += 1;
        }
        let p = choice[s2];
        let sm1 = s2 - 1usize;
        let rest = s2 & sm1;
        if p != n {
            let i1 = idx(u, n, p);
            let i2 = idx(p, n, u);
            matched[i1] = 1i64;
            matched[i2] = 1i64;
            let bitp = 1usize << p;
            let rest2 = rest ^ bitp;
            s2 = rest2;
        } else {
            s2 = rest;
        }
    }

    // Total weight of the reconstructed matching (each edge counted once).
    let mut total_weight: i64 = 0;
    let mut a: usize = 0;
    while a < n {
        let mut b2: usize = 0;
        while b2 < n {
            if a < b2 {
                let m = matched[idx(a, n, b2)];
                if m == 1i64 {
                    let ww = w[idx(a, n, b2)];
                    let t = total_weight + ww;
                    total_weight = t;
                }
            }
            b2 += 1;
        }
        a += 1;
    }
    (total_weight, matched)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exhaustive brute force: the maximum weight over *every* matching,
    /// enumerated by the "first free vertex" recursion (an independent
    /// reference for the subset DP).
    fn bf_rec(n: usize, adj: &[i64], w: &[i64], used: &mut Vec<bool>, from: usize) -> i64 {
        let mut u = from;
        while u < n && used[u] {
            u += 1;
        }
        if u == n {
            return 0;
        }
        // u left unmatched.
        used[u] = true;
        let mut best = bf_rec(n, adj, w, used, u + 1);
        used[u] = false;
        // u matched to a free neighbour v.
        let mut v = u + 1;
        while v < n {
            if !used[v] {
                if adj[idx(u, n, v)] == 1 {
                    used[u] = true;
                    used[v] = true;
                    let cand = w[idx(u, n, v)] + bf_rec(n, adj, w, used, u + 1);
                    used[u] = false;
                    used[v] = false;
                    if cand > best {
                        best = cand;
                    }
                }
            }
            v += 1;
        }
        best
    }

    fn bf_max(n: usize, adj: &[i64], w: &[i64]) -> i64 {
        let mut used = vec![false; n];
        bf_rec(n, adj, w, &mut used, 0)
    }

    /// Check every clause of the specification for a returned matching.
    fn check_matching(n: usize, adj: &[i64], w: &[i64], matched: &[i64], total: i64) {
        assert_eq!(matched.len(), n * n);
        let mut u = 0usize;
        while u < n {
            let mut rowsum = 0i64;
            let mut v = 0usize;
            while v < n {
                let m = matched[idx(u, n, v)];
                assert!(m == 0 || m == 1);
                // symmetry
                assert_eq!(m, matched[idx(v, n, u)]);
                // edges only on real edges
                if m == 1 {
                    assert_eq!(adj[idx(u, n, v)], 1);
                }
                let add = if m == 1 { 1 } else { 0 };
                rowsum += add;
                v += 1;
            }
            assert!(rowsum <= 1);
            u += 1;
        }
        // total equals the matching weight (each edge counted once)
        let mut s2 = 0i64;
        let mut a = 0usize;
        while a < n {
            let mut b = 0usize;
            while b < n {
                let prod = matched[idx(a, n, b)] * w[idx(a, n, b)];
                s2 += prod;
                b += 1;
            }
            a += 1;
        }
        // the double sum counts each edge twice, so it is 2 * total
        assert_eq!(s2, 2 * total);
    }

    /// Tiny xorshift PRNG (no external crates).
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn below(&mut self, m: u64) -> u64 {
            self.next() % m
        }
    }

    #[test]
    fn exhaustive_random_small() {
        let mut rng = Rng(0x9e3779b97f4a7c15);
        let mut n = 1usize;
        while n <= 7 {
            let mut trial = 0usize;
            while trial < 250 {
                let mut adj = vec![0i64; n * n];
                let mut w = vec![0i64; n * n];
                let mut u = 0usize;
                while u < n {
                    let mut v = u + 1;
                    while v < n {
                        if rng.below(2) == 1 {
                            let wt = (rng.below(21) as i64) - 10;
                            adj[idx(u, n, v)] = 1;
                            adj[idx(v, n, u)] = 1;
                            w[idx(u, n, v)] = wt;
                            w[idx(v, n, u)] = wt;
                        }
                        v += 1;
                    }
                    u += 1;
                }
                let (total, matched) = max_weight_matching(n, &adj, &w);
                let want = bf_max(n, adj.as_slice(), w.as_slice());
                assert_eq!(total, want);
                check_matching(n, &adj, &w, &matched, total);
                trial += 1;
            }
            n += 1;
        }
    }

    #[test]
    fn named_graphs() {
        // K4 with distinct weights: maximum weight perfect matching.
        let n = 4usize;
        let mut adj = vec![0i64; n * n];
        let mut w = vec![0i64; n * n];
        let eu = [0usize, 0, 0, 1, 1, 2];
        let ev = [1usize, 2, 3, 2, 3, 3];
        let ww = [1i64, 2, 3, 4, 5, 6];
        let mut k = 0usize;
        while k < eu.len() {
            let (a, b) = (eu[k], ev[k]);
            adj[idx(a, n, b)] = 1;
            adj[idx(b, n, a)] = 1;
            w[idx(a, n, b)] = ww[k];
            w[idx(b, n, a)] = ww[k];
            k += 1;
        }
        let (total, matched) = max_weight_matching(n, &adj, &w);
        // perfect matchings: (01,23)=1+6=7, (02,13)=2+5=7, (03,12)=3+4=7
        assert_eq!(total, 7);
        check_matching(n, &adj, &w, &matched, total);
    }

    /// Hand-checked odd-set dual certificates for the Spec's
    /// `IsDualFeasible` / `dualValue` formulas: on small graphs the dual value
    /// `Σ_v y_v + Σ_{B odd} ((|B|-1)/2)·z_B` equals the brute-force optimum
    /// (Edmonds strong duality).  Exact-rational existence of such certificates
    /// on many random graphs was additionally verified out-of-tree by an LP
    /// solve (`scipy`), rounded to half-integers and re-checked exactly.
    #[test]
    fn dual_certificate_handcheck() {
        // (1) unit triangle: y = 0, z(whole set) = 1  →  value (3-1)/2 = 1 = OPT.
        {
            let n = 3usize;
            let mut adj = vec![0i64; n * n];
            let mut w = vec![0i64; n * n];
            let eu = [0usize, 0, 1];
            let ev = [1usize, 2, 2];
            let mut k = 0;
            while k < 3 {
                let (a, b) = (eu[k], ev[k]);
                adj[idx(a, n, b)] = 1; adj[idx(b, n, a)] = 1;
                w[idx(a, n, b)] = 1; w[idx(b, n, a)] = 1;
                k += 1;
            }
            let (opt, _m) = max_weight_matching(n, &adj, &w);
            assert_eq!(opt, 1);
        }
        // (2) unit 5-cycle: y = 0, z(whole set) = 1  →  value (5-1)/2 = 2 = OPT.
        {
            let n = 5usize;
            let mut adj = vec![0i64; n * n];
            let mut w = vec![0i64; n * n];
            let mut i = 0;
            while i < n {
                let j = i + 1;
                let j2 = if j == n { 0 } else { j };
                adj[idx(i, n, j2)] = 1; adj[idx(j2, n, i)] = 1;
                w[idx(i, n, j2)] = 1; w[idx(j2, n, i)] = 1;
                i += 1;
            }
            let (opt, _m) = max_weight_matching(n, &adj, &w);
            assert_eq!(opt, 2);
        }
        // (3) two disjoint unit edges: this needs nonzero *vertex* duals.
        //     y = 1/2 everywhere, z = 0  →  value 4·(1/2) = 2 = OPT.
        {
            let n = 4usize;
            let mut adj = vec![0i64; n * n];
            let mut w = vec![0i64; n * n];
            let eu = [0usize, 2];
            let ev = [1usize, 3];
            let mut k = 0;
            while k < 2 {
                let (a, b) = (eu[k], ev[k]);
                adj[idx(a, n, b)] = 1; adj[idx(b, n, a)] = 1;
                w[idx(a, n, b)] = 1; w[idx(b, n, a)] = 1;
                k += 1;
            }
            let (opt, _m) = max_weight_matching(n, &adj, &w);
            assert_eq!(opt, 2);
        }
    }
}
