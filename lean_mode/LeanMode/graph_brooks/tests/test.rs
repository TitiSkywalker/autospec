//! Tests for `brooks_color`.
//!
//! Every test checks the three things the API promises:
//!   (a) `col.len() == n`,
//!   (b) `col` is a *proper* colouring of the adjacency matrix,
//!   (c) the number of colours used is `≤ Δ + 1` always, and `≤ Δ` whenever
//!       the graph is connected, not complete and not an odd cycle.
//!
//! The helper functions below are deliberately named with a `t_` prefix so
//! that they cannot collide with the (private) helpers of `src/lib.rs`.

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
        let i1 = u * n;
        let i2 = i1 + v;
        let j1 = v * n;
        let j2 = j1 + u;
        adj[i2] = 1;
        adj[j2] = 1;
        k += 1;
    }
    adj
}

/// Is `col` a proper colouring of `adj`?
fn t_proper(n: usize, adj: &[usize], col: &[usize]) -> bool {
    let mut ok: bool = true;
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = u + 1;
        while v < n {
            let a = adj[t_idx(u, n, v)];
            if a == 1 {
                let cu = col[u];
                let cv = col[v];
                if cu == cv {
                    ok = false;
                }
            }
            v += 1;
        }
        u += 1;
    }
    ok
}

/// Number of distinct colours occurring in `col[0..n]`.
fn t_used(col: &[usize], n: usize) -> usize {
    let mut mark: Vec<usize> = vec![0usize; n + 1];
    let mut i: usize = 0;
    while i < n {
        let c = col[i];
        mark[c] = 1;
        i += 1;
    }
    let mut cnt: usize = 0;
    let mut j: usize = 0;
    while j <= n {
        let m = mark[j];
        if m == 1 {
            cnt += 1;
        }
        j += 1;
    }
    cnt
}

/// Maximum degree `Δ` of the graph.
fn t_delta(n: usize, adj: &[usize]) -> usize {
    let mut d: usize = 0;
    let mut v: usize = 0;
    while v < n {
        let mut deg: usize = 0;
        let mut u: usize = 0;
        while u < n {
            let a = adj[t_idx(u, n, v)];
            if a == 1 {
                deg += 1;
            }
            u += 1;
        }
        if deg > d {
            d = deg;
        }
        v += 1;
    }
    d
}

/// `Δ + 1`.
fn t_delta_plus_one(n: usize, adj: &[usize]) -> usize {
    let d = t_delta(n, adj);
    let r = d + 1;
    r
}

/// Number of colours used by plain first-fit greedy in the order `0..n`
/// (i.e. without the Brooks ordering and with an unbounded palette).
fn t_first_fit_used(n: usize, adj: &[usize]) -> usize {
    let mut col: Vec<usize> = vec![0usize; n];
    let mut mark: Vec<usize> = vec![0usize; n + 1];
    let mut v: usize = 0;
    while v < n {
        let mut t: usize = 0;
        while t <= n {
            mark[t] = 0;
            t += 1;
        }
        let mut u: usize = 0;
        while u < v {
            let a = adj[t_idx(u, n, v)];
            if a == 1 {
                let cu = col[u];
                mark[cu] = 1;
            }
            u += 1;
        }
        let mut c: usize = n + 1;
        let mut t2: usize = 0;
        while t2 <= n {
            let m = mark[t2];
            if m == 0 {
                if t2 < c {
                    c = t2;
                }
            }
            t2 += 1;
        }
        col[v] = c;
        v += 1;
    }
    t_used(&col, n)
}

// ── degenerate sizes ────────────────────────────────────────────────────────

#[test]
fn test_n_zero() {
    let col = graph_brooks::brooks_color(0, &[]);
    assert_eq!(col.len(), 0);
    let used = t_used(&col, 0);
    assert_eq!(used, 0);
}

#[test]
fn test_n_one() {
    let col = graph_brooks::brooks_color(1, &[]);
    assert_eq!(col.len(), 1);
    assert_eq!(col[0], 0);
    let proper = t_proper(1, &[], &col);
    assert!(proper);
    let used = t_used(&col, 1);
    assert_eq!(used, 1);
}

#[test]
fn test_no_edges() {
    // n = 4, no edges: Δ = 0, one colour suffices
    let eu: Vec<usize> = vec![];
    let ev: Vec<usize> = vec![];
    let adj = build_adj(4, &eu, &ev);
    let col = graph_brooks::brooks_color(4, &adj);
    assert_eq!(col.len(), 4);
    let proper = t_proper(4, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 4);
    let d1 = t_delta_plus_one(4, &adj);
    assert!(used <= d1);
    assert_eq!(used, 1);
}

// ── paths and cycles (Δ = 2) ────────────────────────────────────────────────

#[test]
fn test_path_four() {
    // path 0 - 1 - 2 - 3, Δ = 2
    let eu = vec![0, 1, 2];
    let ev = vec![1, 2, 3];
    let adj = build_adj(4, &eu, &ev);
    let col = graph_brooks::brooks_color(4, &adj);
    assert_eq!(col.len(), 4);
    let proper = t_proper(4, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 4);
    let d1 = t_delta_plus_one(4, &adj);
    assert!(used <= d1);
    let d = t_delta(4, &adj);
    assert!(used <= d); // Brooks: connected, not complete, not an odd cycle
    assert_eq!(used, 2);
}

#[test]
fn test_even_cycle_c4() {
    // 4-cycle 0 - 1 - 2 - 3 - 0, Δ = 2
    let eu = vec![0, 1, 2, 3];
    let ev = vec![1, 2, 3, 0];
    let adj = build_adj(4, &eu, &ev);
    let col = graph_brooks::brooks_color(4, &adj);
    assert_eq!(col.len(), 4);
    let proper = t_proper(4, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 4);
    let d1 = t_delta_plus_one(4, &adj);
    assert!(used <= d1);
    let d = t_delta(4, &adj);
    assert!(used <= d);
    assert_eq!(used, 2);
}

#[test]
fn test_even_cycle_c6() {
    // 6-cycle 0 - 1 - ... - 5 - 0, Δ = 2
    let eu = vec![0, 1, 2, 3, 4, 5];
    let ev = vec![1, 2, 3, 4, 5, 0];
    let adj = build_adj(6, &eu, &ev);
    let col = graph_brooks::brooks_color(6, &adj);
    assert_eq!(col.len(), 6);
    let proper = t_proper(6, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 6);
    let d1 = t_delta_plus_one(6, &adj);
    assert!(used <= d1);
    let d = t_delta(6, &adj);
    assert!(used <= d);
    assert_eq!(used, 2);
}

#[test]
fn test_odd_cycle_c3() {
    // triangle: complete, so Δ + 1 = 3 colours are needed
    let eu = vec![0, 1, 2];
    let ev = vec![1, 2, 0];
    let adj = build_adj(3, &eu, &ev);
    let col = graph_brooks::brooks_color(3, &adj);
    assert_eq!(col.len(), 3);
    let proper = t_proper(3, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 3);
    let d1 = t_delta_plus_one(3, &adj);
    assert!(used <= d1);
    assert_eq!(used, 3);
}

#[test]
fn test_odd_cycle_c5() {
    // 5-cycle, Δ = 2, needs 3 = Δ + 1 colours
    let eu = vec![0, 1, 2, 3, 4];
    let ev = vec![1, 2, 3, 4, 0];
    let adj = build_adj(5, &eu, &ev);
    let col = graph_brooks::brooks_color(5, &adj);
    assert_eq!(col.len(), 5);
    let proper = t_proper(5, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 5);
    let d1 = t_delta_plus_one(5, &adj);
    assert!(used <= d1);
    assert_eq!(used, 3);
}

#[test]
fn test_odd_cycle_c7() {
    // 7-cycle, Δ = 2, needs 3 = Δ + 1 colours
    let eu = vec![0, 1, 2, 3, 4, 5, 6];
    let ev = vec![1, 2, 3, 4, 5, 6, 0];
    let adj = build_adj(7, &eu, &ev);
    let col = graph_brooks::brooks_color(7, &adj);
    assert_eq!(col.len(), 7);
    let proper = t_proper(7, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 7);
    let d1 = t_delta_plus_one(7, &adj);
    assert!(used <= d1);
    assert_eq!(used, 3);
}

// ── complete graphs (the Δ + 1 extremal case) ───────────────────────────────

#[test]
fn test_complete_k4() {
    // K4: Δ = 3, needs 4 = Δ + 1 colours
    let eu = vec![0, 0, 0, 1, 1, 2];
    let ev = vec![1, 2, 3, 2, 3, 3];
    let adj = build_adj(4, &eu, &ev);
    let col = graph_brooks::brooks_color(4, &adj);
    assert_eq!(col.len(), 4);
    let proper = t_proper(4, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 4);
    let d1 = t_delta_plus_one(4, &adj);
    assert!(used <= d1);
    assert_eq!(used, 4);
}

#[test]
fn test_complete_k5() {
    // K5: Δ = 4, needs 5 = Δ + 1 colours
    let eu = vec![0, 0, 0, 0, 1, 1, 1, 2, 2, 3];
    let ev = vec![1, 2, 3, 4, 2, 3, 4, 3, 4, 4];
    let adj = build_adj(5, &eu, &ev);
    let col = graph_brooks::brooks_color(5, &adj);
    assert_eq!(col.len(), 5);
    let proper = t_proper(5, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 5);
    let d1 = t_delta_plus_one(5, &adj);
    assert!(used <= d1);
    assert_eq!(used, 5);
}

// ── Brooks: Δ colours suffice ───────────────────────────────────────────────

#[test]
fn test_k4_minus_edge() {
    // K4 without the edge {2,3}: Δ = 3, χ = 3 = Δ
    let eu = vec![0, 0, 0, 1, 1];
    let ev = vec![1, 2, 3, 2, 3];
    let adj = build_adj(4, &eu, &ev);
    let col = graph_brooks::brooks_color(4, &adj);
    assert_eq!(col.len(), 4);
    let proper = t_proper(4, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 4);
    let d1 = t_delta_plus_one(4, &adj);
    assert!(used <= d1);
    let d = t_delta(4, &adj);
    assert!(used <= d);
    assert_eq!(used, 3);
}

#[test]
fn test_complete_bipartite_k33() {
    // K_{3,3}: Δ = 3, but 2 colours suffice
    let eu = vec![0, 0, 0, 1, 1, 1, 2, 2, 2];
    let ev = vec![3, 4, 5, 3, 4, 5, 3, 4, 5];
    let adj = build_adj(6, &eu, &ev);
    let col = graph_brooks::brooks_color(6, &adj);
    assert_eq!(col.len(), 6);
    let proper = t_proper(6, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 6);
    let d1 = t_delta_plus_one(6, &adj);
    assert!(used <= d1);
    let d = t_delta(6, &adj);
    assert!(used <= d);
    assert_eq!(used, 2);
}

#[test]
fn test_star() {
    // star K_{1,3}: Δ = 3, 2 colours suffice
    let eu = vec![0, 0, 0];
    let ev = vec![1, 2, 3];
    let adj = build_adj(4, &eu, &ev);
    let col = graph_brooks::brooks_color(4, &adj);
    assert_eq!(col.len(), 4);
    let proper = t_proper(4, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 4);
    let d1 = t_delta_plus_one(4, &adj);
    assert!(used <= d1);
    let d = t_delta(4, &adj);
    assert!(used <= d);
    assert_eq!(used, 2);
}

#[test]
fn test_bowtie_cut_vertex() {
    // bowtie: triangles 0-1-2 and 0-3-4 sharing the cut vertex 0,
    // Δ = 4 (deg 0 = 4), χ = 3 ≤ Δ
    let eu = vec![0, 1, 2, 0, 3, 4];
    let ev = vec![1, 2, 0, 3, 4, 0];
    let adj = build_adj(5, &eu, &ev);
    let col = graph_brooks::brooks_color(5, &adj);
    assert_eq!(col.len(), 5);
    let proper = t_proper(5, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 5);
    let d1 = t_delta_plus_one(5, &adj);
    assert!(used <= d1);
    let d = t_delta(5, &adj);
    assert!(used <= d);
    assert_eq!(used, 3);
}

#[test]
fn test_petersen_needs_brooks_order() {
    // Petersen graph (Δ = 3, χ = 3), labelled so that plain first-fit greedy
    // in the order 0..9 needs 4 > Δ colours.  brooks_color must still find a
    // proper 3-colouring, which requires the Brooks ordering of step 3.
    let eu = vec![0, 0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 4, 4, 5, 7];
    let ev = vec![1, 3, 8, 6, 9, 6, 7, 8, 5, 7, 5, 8, 9, 6, 9];
    let adj = build_adj(10, &eu, &ev);
    let col = graph_brooks::brooks_color(10, &adj);
    assert_eq!(col.len(), 10);
    let proper = t_proper(10, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 10);
    let d1 = t_delta_plus_one(10, &adj);
    assert!(used <= d1);
    let d = t_delta(10, &adj);
    assert!(used <= d);
    assert_eq!(used, 3);
    // plain greedy is not good enough here: it wastes a Δ + 1-st colour
    let greedy = t_first_fit_used(10, &adj);
    assert!(greedy > d);
    assert_eq!(greedy, 4);
}

// ── disconnected graphs ─────────────────────────────────────────────────────

#[test]
fn test_disconnected_two_edges() {
    // edges {0,1} and {2,3}: Δ = 1, two colours
    let eu = vec![0, 2];
    let ev = vec![1, 3];
    let adj = build_adj(4, &eu, &ev);
    let col = graph_brooks::brooks_color(4, &adj);
    assert_eq!(col.len(), 4);
    let proper = t_proper(4, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 4);
    let d1 = t_delta_plus_one(4, &adj);
    assert!(used <= d1);
    assert_eq!(used, 2);
}

#[test]
fn test_disconnected_two_triangles() {
    // two disjoint triangles: Δ = 2, three colours (≠ Δ + 1 still holds)
    let eu = vec![0, 1, 2, 3, 4, 5];
    let ev = vec![1, 2, 0, 4, 5, 3];
    let adj = build_adj(6, &eu, &ev);
    let col = graph_brooks::brooks_color(6, &adj);
    assert_eq!(col.len(), 6);
    let proper = t_proper(6, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 6);
    let d1 = t_delta_plus_one(6, &adj);
    assert!(used <= d1);
    assert_eq!(used, 3);
}

#[test]
fn test_k4_plus_isolated() {
    // K4 plus an isolated vertex: Δ = 3, disconnected, so the Δ bound does
    // not apply by Brooks; Δ + 1 = 4 colours are used (and are necessary).
    let eu = vec![0, 0, 0, 1, 1, 2];
    let ev = vec![1, 2, 3, 2, 3, 3];
    let adj = build_adj(5, &eu, &ev);
    let col = graph_brooks::brooks_color(5, &adj);
    assert_eq!(col.len(), 5);
    let proper = t_proper(5, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 5);
    let d1 = t_delta_plus_one(5, &adj);
    assert!(used <= d1);
    assert_eq!(used, 4);
}

#[test]
fn test_two_disjoint_k4_minus_edge() {
    // two disjoint copies of K4 minus an edge: Δ = 3 and χ = 3, but the graph
    // is disconnected, so the Brooks-ordering search finds no vertex pair and
    // the backtracking search has to produce the 3-colouring.
    let eu = vec![0, 0, 0, 1, 1, 4, 4, 4, 5, 5];
    let ev = vec![1, 2, 3, 2, 3, 5, 6, 7, 6, 7];
    let adj = build_adj(8, &eu, &ev);
    let col = graph_brooks::brooks_color(8, &adj);
    assert_eq!(col.len(), 8);
    let proper = t_proper(8, &adj, &col);
    assert!(proper);
    let used = t_used(&col, 8);
    let d1 = t_delta_plus_one(8, &adj);
    assert!(used <= d1);
    assert_eq!(used, 3);
}
