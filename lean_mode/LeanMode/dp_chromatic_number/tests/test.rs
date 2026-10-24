//! Tests for `chromatic_number`.
//!
//! Helpers are named with a `t_`/`bf_`/`build_` prefix so they cannot collide
//! with (or prefix-match) the private helpers of `src/lib.rs`.

/// Flat row-major index (test-local copy of the implementation's `idx`).
fn t_idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Build a flat row-major `n × n` symmetric 0/1 adjacency matrix from the
/// undirected edge list `(eu[k], ev[k])`.
fn build_adj(n: usize, eu: &[usize], ev: &[usize]) -> Vec<usize> {
    let mut adj: Vec<usize> = vec![0usize; n * n];
    let m = eu.len();
    let mut k: usize = 0;
    while k < m {
        let u = eu[k];
        let v = ev[k];
        let i2 = t_idx(u, n, v);
        let j2 = t_idx(v, n, u);
        adj[i2] = 1;
        adj[j2] = 1;
        k += 1;
    }
    adj
}

// ── Independent brute force: count proper k-colourings, then scan k ─────────

/// Number of proper colourings of `v..n` (vertices `0..v` already coloured in
/// `colors`) with the palette `0..k`, given the vertex `v` is next to colour.
fn bf_count(adj: &[usize], n: usize, colors: &mut Vec<usize>, v: usize, k: usize) -> usize {
    if v == n {
        return 1;
    }
    let mut total: usize = 0;
    let mut c: usize = 0;
    while c < k {
        let mut blocked: usize = 0;
        let mut u: usize = 0;
        while u < v {
            let cu = colors[u];
            let a = adj[t_idx(u, n, v)];
            let same: usize = if cu == c { 1 } else { 0 };
            let isedge: usize = if a != 0 { 1 } else { 0 };
            let contrib: usize = same * isedge;
            let blocked2 = blocked + contrib;
            blocked = blocked2;
            u += 1;
        }
        if blocked == 0 {
            colors[v] = c;
            let sub = bf_count(adj, n, colors, v + 1, k);
            let total2 = total + sub;
            total = total2;
        }
        c += 1;
    }
    total
}

/// Brute-force chromatic number: the smallest `k` in `1..=n` admitting a
/// proper `k`-colouring (`n` colours always suffice, one per vertex).
fn bf_chromatic(n: usize, adj: &[usize]) -> usize {
    let mut ans: usize = n;
    let mut k: usize = n;
    while k > 0 {
        let mut colors: Vec<usize> = vec![0usize; n];
        let cnt = bf_count(adj, n, &mut colors, 0, k);
        if cnt != 0 {
            ans = k;
        }
        k -= 1;
    }
    ans
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn test_empty_vertex_set() {
    let adj: Vec<usize> = vec![];
    assert_eq!(dp_chromatic_number::chromatic_number(0, &adj), 0);
}

#[test]
fn test_single_vertex() {
    let adj = vec![0usize];
    assert_eq!(dp_chromatic_number::chromatic_number(1, &adj), 1);
}

#[test]
fn test_no_edges() {
    // Any positive number of vertices with no edges needs one colour.
    let adj = vec![0usize; 36];
    assert_eq!(dp_chromatic_number::chromatic_number(6, &adj), 1);
}

#[test]
fn test_single_edge() {
    let adj = build_adj(2, &[0], &[1]);
    assert_eq!(dp_chromatic_number::chromatic_number(2, &adj), 2);
}

// ── Known-answer graphs ─────────────────────────────────────────────────────

#[test]
fn test_paths() {
    // A path on ≥ 2 vertices is bipartite.
    let adj = build_adj(5, &[0, 1, 2, 3], &[1, 2, 3, 4]);
    assert_eq!(dp_chromatic_number::chromatic_number(5, &adj), 2);
}

#[test]
fn test_even_cycles() {
    let c4 = build_adj(4, &[0, 1, 2, 3], &[1, 2, 3, 0]);
    assert_eq!(dp_chromatic_number::chromatic_number(4, &c4), 2);
    let c6 = build_adj(6, &[0, 1, 2, 3, 4, 5], &[1, 2, 3, 4, 5, 0]);
    assert_eq!(dp_chromatic_number::chromatic_number(6, &c6), 2);
}

#[test]
fn test_odd_cycles() {
    let c3 = build_adj(3, &[0, 1, 2], &[1, 2, 0]);
    assert_eq!(dp_chromatic_number::chromatic_number(3, &c3), 3);
    let c5 = build_adj(5, &[0, 1, 2, 3, 4], &[1, 2, 3, 4, 0]);
    assert_eq!(dp_chromatic_number::chromatic_number(5, &c5), 3);
    let c7 = build_adj(7, &[0, 1, 2, 3, 4, 5, 6], &[1, 2, 3, 4, 5, 6, 0]);
    assert_eq!(dp_chromatic_number::chromatic_number(7, &c7), 3);
}

#[test]
fn test_complete_graphs() {
    let k3 = build_adj(3, &[0, 0, 1], &[1, 2, 2]);
    assert_eq!(dp_chromatic_number::chromatic_number(3, &k3), 3);
    let k4 = build_adj(4, &[0, 0, 0, 1, 1, 2], &[1, 2, 3, 2, 3, 3]);
    assert_eq!(dp_chromatic_number::chromatic_number(4, &k4), 4);
    let k5 = build_adj(5, &[0, 0, 0, 0, 1, 1, 1, 2, 2, 3], &[1, 2, 3, 4, 2, 3, 4, 3, 4, 4]);
    assert_eq!(dp_chromatic_number::chromatic_number(5, &k5), 5);
}

#[test]
fn test_complete_bipartite() {
    // K_{2,3}: parts {0,1} and {2,3,4}.
    let adj = build_adj(5, &[0, 0, 0, 1, 1, 1], &[2, 3, 4, 2, 3, 4]);
    assert_eq!(dp_chromatic_number::chromatic_number(5, &adj), 2);
}

#[test]
fn test_star() {
    // Star K_{1,5}: centre 0 joined to 1..5 — a tree, hence bipartite.
    let adj = build_adj(6, &[0, 0, 0, 0, 0], &[1, 2, 3, 4, 5]);
    assert_eq!(dp_chromatic_number::chromatic_number(6, &adj), 2);
}

#[test]
fn test_wheel_needs_four() {
    // Wheel W6 = a 5-cycle plus a hub joined to every cycle vertex.  The
    // 5-cycle forces 3 colours on the rim and the hub forbids all of them.
    let adj = build_adj(
        6,
        &[0, 1, 2, 3, 4, 5, 5, 5, 5, 5],
        &[1, 2, 3, 4, 0, 0, 1, 2, 3, 4],
    );
    assert_eq!(dp_chromatic_number::chromatic_number(6, &adj), 4);
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_with_brute_force_small() {
    // Enumerate *every* graph on 1..=4 vertices and compare with the brute force.
    let mut n: usize = 1;
    while n <= 4 {
        let p = n * (n - 1) / 2;
        let total = 1usize << p;
        let mut em: usize = 0;
        while em < total {
            let mut adj: Vec<usize> = vec![0usize; n * n];
            let mut bit: usize = 0;
            let mut u: usize = 0;
            while u < n {
                let mut v: usize = u + 1;
                while v < n {
                    let on = (em >> bit) & 1usize;
                    if on != 0 {
                        let i2 = t_idx(u, n, v);
                        let j2 = t_idx(v, n, u);
                        adj[i2] = 1;
                        adj[j2] = 1;
                    }
                    bit += 1;
                    v += 1;
                }
                u += 1;
            }
            let got = dp_chromatic_number::chromatic_number(n, &adj);
            let want = bf_chromatic(n, &adj);
            assert_eq!(got, want);
            em += 1;
        }
        n += 1;
    }
}

#[test]
fn test_agrees_with_brute_force_named() {
    // A few larger named graphs, checked against the brute force.
    let g1 = build_adj(5, &[0, 1, 2, 3, 4], &[1, 2, 3, 4, 0]); // C5
    assert_eq!(
        dp_chromatic_number::chromatic_number(5, &g1),
        bf_chromatic(5, &g1)
    );
    let g2 = build_adj(6, &[0, 1, 2, 3, 4, 5, 5, 5, 5, 5], &[1, 2, 3, 4, 0, 0, 1, 2, 3, 4]); // W6
    assert_eq!(
        dp_chromatic_number::chromatic_number(6, &g2),
        bf_chromatic(6, &g2)
    );
}
