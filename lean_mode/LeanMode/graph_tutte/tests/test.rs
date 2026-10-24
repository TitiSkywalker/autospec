// Tests for Tutte's 1-factor condition.
//
// Every graph is given as a flat, row-major `n × n` 0/1 symmetric adjacency
// matrix.  Each small case is cross-checked against a brute-force
// perfect-matching search written below (note: the helper names are prefixed
// `bf_`/`tt_` so they never collide with anything in `src/lib.rs`).

/// Test-local flat index for a row-major `n × n` matrix.
fn tt_adj(adj: &[usize], n: usize, u: usize, v: usize) -> usize {
    let t = u * n;
    let r = t + v;
    adj[r]
}

/// Bit `k` of `mask` (`0` or `1`), via doubling + `/` + `%`.
fn tt_bit(mask: usize, k: usize) -> usize {
    let mut p: usize = 1;
    let mut i: usize = 0;
    while i < k {
        p = p * 2;
        i += 1;
    }
    let q = mask / p;
    let b = q % 2;
    b
}

/// Brute-force perfect matching: enumerate every subset of the edge set and
/// check whether one of them is a matching that covers all `n` vertices.
fn bf_perfect_matching(n: usize, adj: &[usize]) -> bool {
    // Collect the edges {u, v} with u < v.
    let mut eu: Vec<usize> = Vec::new();
    let mut ev: Vec<usize> = Vec::new();
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = 0;
        while v < n {
            if v > u {
                let a = tt_adj(adj, n, u, v);
                if a == 1 {
                    eu.push(u);
                    ev.push(v);
                }
            }
            v += 1;
        }
        u += 1;
    }
    let m = eu.len();

    // limit = 2^m
    let mut limit: usize = 1;
    let mut e: usize = 0;
    while e < m {
        limit = limit * 2;
        e += 1;
    }

    let mut ok: bool = false;
    let mut mask: usize = 0;
    while mask < limit {
        let mut used: Vec<usize> = vec![0usize; n];
        let mut good: bool = true;
        let mut k: usize = 0;
        while k < m {
            let b = tt_bit(mask, k);
            if b == 1 {
                let x = eu[k];
                let y = ev[k];
                if used[x] == 1 {
                    good = false;
                } else {
                    if used[y] == 1 {
                        good = false;
                    } else {
                        used[x] = 1;
                        used[y] = 1;
                    }
                }
            }
            k += 1;
        }
        if good {
            let mut cnt: usize = 0;
            let mut q: usize = 0;
            while q < n {
                if used[q] == 1 {
                    cnt += 1;
                }
                q += 1;
            }
            if cnt == n {
                ok = true;
            }
        }
        mask += 1;
    }
    ok
}

#[test]
fn test_empty_graph_3() {
    // 3 isolated vertices: S = ∅ already gives 3 odd components > 0.
    let adj = vec![
        0, 0, 0,
        0, 0, 0,
        0, 0, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(3, &adj);
    assert_eq!(r, false);
    let bf = bf_perfect_matching(3, &adj);
    assert_eq!(r, bf);
}

#[test]
fn test_k2() {
    // Single edge: perfect matching.
    let adj = vec![
        0, 1,
        1, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(2, &adj);
    assert_eq!(r, true);
    assert_eq!(r, bf_perfect_matching(2, &adj));
}

#[test]
fn test_k4() {
    // Complete graph K4.
    let adj = vec![
        0, 1, 1, 1,
        1, 0, 1, 1,
        1, 1, 0, 1,
        1, 1, 1, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(4, &adj);
    assert_eq!(r, true);
    assert_eq!(r, bf_perfect_matching(4, &adj));
}

#[test]
fn test_path_p4() {
    // Path 0 - 1 - 2 - 3.
    let adj = vec![
        0, 1, 0, 0,
        1, 0, 1, 0,
        0, 1, 0, 1,
        0, 0, 1, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(4, &adj);
    assert_eq!(r, true);
    assert_eq!(r, bf_perfect_matching(4, &adj));
}

#[test]
fn test_path_p3() {
    // Path 0 - 1 - 2 on an odd number of vertices: no perfect matching.
    let adj = vec![
        0, 1, 0,
        1, 0, 1,
        0, 1, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(3, &adj);
    assert_eq!(r, false);
    assert_eq!(r, bf_perfect_matching(3, &adj));
}

#[test]
fn test_c6() {
    // Cycle on 6 vertices.
    let adj = vec![
        0, 1, 0, 0, 0, 1,
        1, 0, 1, 0, 0, 0,
        0, 1, 0, 1, 0, 0,
        0, 0, 1, 0, 1, 0,
        0, 0, 0, 1, 0, 1,
        1, 0, 0, 0, 1, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(6, &adj);
    assert_eq!(r, true);
    assert_eq!(r, bf_perfect_matching(6, &adj));
}

#[test]
fn test_c5() {
    // Odd cycle: S = ∅ gives one odd component > 0.
    let adj = vec![
        0, 1, 0, 0, 1,
        1, 0, 1, 0, 0,
        0, 1, 0, 1, 0,
        0, 0, 1, 0, 1,
        1, 0, 0, 1, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(5, &adj);
    assert_eq!(r, false);
    assert_eq!(r, bf_perfect_matching(5, &adj));
}

#[test]
fn test_triangle_plus_isolated() {
    // Triangle 0-1-2 plus the isolated vertex 3.
    let adj = vec![
        0, 1, 1, 0,
        1, 0, 1, 0,
        1, 1, 0, 0,
        0, 0, 0, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(4, &adj);
    assert_eq!(r, false);
    assert_eq!(r, bf_perfect_matching(4, &adj));
}

#[test]
fn test_two_triangles() {
    // Two disjoint triangles C3 ⊔ C3 (6 vertices, even).  Each triangle has
    // odd order and is an independent component, so neither admits a perfect
    // matching; with S = ∅ there are two odd components > |S| = 0, so Tutte's
    // condition FAILS even though the order is even.  This is the classical
    // counterexample showing "regular of even order" alone does not imply a
    // 1-factor.
    let adj = vec![
        0, 1, 1, 0, 0, 0,
        1, 0, 1, 0, 0, 0,
        1, 1, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 1,
        0, 0, 0, 1, 0, 1,
        0, 0, 0, 1, 1, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(6, &adj);
    assert_eq!(r, false);
    assert_eq!(r, bf_perfect_matching(6, &adj));
}

#[test]
fn test_star_k13() {
    // Star K_{1,3}: removing the centre leaves 3 odd components > 1.
    let adj = vec![
        0, 1, 1, 1,
        1, 0, 0, 0,
        1, 0, 0, 0,
        1, 0, 0, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(4, &adj);
    assert_eq!(r, false);
    assert_eq!(r, bf_perfect_matching(4, &adj));
}

#[test]
fn test_k33() {
    // Complete bipartite K_{3,3}: non-complete, and Tutte's condition holds.
    let adj = vec![
        0, 0, 0, 1, 1, 1,
        0, 0, 0, 1, 1, 1,
        0, 0, 0, 1, 1, 1,
        1, 1, 1, 0, 0, 0,
        1, 1, 1, 0, 0, 0,
        1, 1, 1, 0, 0, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(6, &adj);
    assert_eq!(r, true);
    assert_eq!(r, bf_perfect_matching(6, &adj));
}

#[test]
fn test_petersen() {
    // The Petersen graph (Petersen, 1891): the canonical 3-regular graph on 10
    // vertices.  It is not complete, yet it satisfies Tutte's condition (it has
    // a perfect matching) — the standard example for the corollary that every
    // bridgeless cubic graph has a 1-factor.
    let adj = vec![
        0, 1, 0, 0, 1, 1, 0, 0, 0, 0,
        1, 0, 1, 0, 0, 0, 1, 0, 0, 0,
        0, 1, 0, 1, 0, 0, 0, 1, 0, 0,
        0, 0, 1, 0, 1, 0, 0, 0, 1, 0,
        1, 0, 0, 1, 0, 0, 0, 0, 0, 1,
        1, 0, 0, 0, 0, 0, 0, 1, 1, 0,
        0, 1, 0, 0, 0, 0, 0, 0, 1, 1,
        0, 0, 1, 0, 0, 1, 0, 0, 0, 1,
        0, 0, 0, 1, 0, 1, 1, 0, 0, 0,
        0, 0, 0, 0, 1, 0, 1, 1, 0, 0,
    ];
    let r = graph_tutte::tutte_condition_holds(10, &adj);
    assert_eq!(r, true);
}
