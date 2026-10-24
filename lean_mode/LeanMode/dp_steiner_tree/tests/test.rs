// ── Independent brute force: enumerate every subset of edges ────────────────

/// `1` when every terminal is still unmarked, `0` when all of them are reached.
fn all_marked(mark: &[usize], terminals: &[usize]) -> usize {
    let mut bad: usize = 0;
    let mut j: usize = 0;
    while j < terminals.len() {
        let t = terminals[j];
        if mark[t] == 0 {
            bad = 1;
        }
        j += 1;
    }
    bad
}

/// Minimum Steiner-tree cost by exhaustive search: try every subset of the
/// `n(n-1)/2` edges of the complete graph and keep the cheapest one that
/// connects all terminals.
fn brute_steiner(n: usize, w: &[usize], terminals: &[usize]) -> usize {
    if terminals.len() < 2 {
        return 0;
    }
    let mut total: usize = 0;
    let mut i: usize = 0;
    while i < n * n {
        total += w[i];
        i += 1;
    }
    let sentinel = total + 1;

    // The edge list (u, v) with u < v, as two parallel Vecs.
    let mut eu: Vec<usize> = Vec::new();
    let mut ev: Vec<usize> = Vec::new();
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = u + 1;
        while v < n {
            eu.push(u);
            ev.push(v);
            v += 1;
        }
        u += 1;
    }
    let m = eu.len();

    let mut best = sentinel;
    let nsubs = 1usize << m;
    let mut mask: usize = 0;
    while mask < nsubs {
        // Weight of the chosen edge set.
        let mut cost: usize = 0;
        let mut e: usize = 0;
        while e < m {
            let bit = 1usize << e;
            let has = mask & bit;
            if has != 0 {
                let a1 = eu[e];
                let b1 = ev[e];
                let c = w[a1 * n + b1];
                cost += c;
            }
            e += 1;
        }
        // Flood the chosen edge set from the first terminal.
        let mut mark = vec![0usize; n];
        let first = terminals[0];
        mark[first] = 1;
        let mut round: usize = 0;
        while round < n {
            let mut e2: usize = 0;
            while e2 < m {
                let bit = 1usize << e2;
                let has = mask & bit;
                if has != 0 {
                    let a1 = eu[e2];
                    let b1 = ev[e2];
                    if mark[a1] == 1 {
                        mark[b1] = 1;
                    }
                    if mark[b1] == 1 {
                        mark[a1] = 1;
                    }
                }
                e2 += 1;
            }
            round += 1;
        }
        let bad = all_marked(&mark, terminals);
        if bad == 0 {
            if cost < best {
                best = cost;
            }
        }
        mask += 1;
    }
    best
}

// ── Degenerate inputs ───────────────────────────────────────────────────────

#[test]
fn test_no_terminals() {
    let w = vec![0, 1, 1, 0];
    let empty: Vec<usize> = vec![];
    assert_eq!(dp_steiner_tree::steiner_tree(2, &w, &empty), 0);
}

#[test]
fn test_single_terminal() {
    let w = vec![0, 1, 1, 0];
    let one = vec![0];
    assert_eq!(dp_steiner_tree::steiner_tree(2, &w, &one), 0);
}

// ── Known-answer tests ─────────────────────────────────────────────────────

#[test]
fn test_direct_edge() {
    // Two terminals joined by a cheap direct edge.
    let w = vec![0, 3, 3, 0];
    let t = vec![0, 1];
    assert_eq!(dp_steiner_tree::steiner_tree(2, &w, &t), 3);
    assert_eq!(brute_steiner(2, &w, &t), 3);
}

#[test]
fn test_detour_through_non_terminal() {
    // Connecting 1 and 2 directly costs 5, but the detour 1–0–2 costs 1 + 1.
    // Vertex 0 is *not* a terminal, yet the algorithm must route through it.
    let w = vec![0, 1, 1, 1, 0, 5, 1, 5, 0];
    let t = vec![1, 2];
    assert_eq!(dp_steiner_tree::steiner_tree(3, &w, &t), 2);
    assert_eq!(brute_steiner(3, &w, &t), 2);
}

#[test]
fn test_triangle_of_terminals() {
    // All three vertices are terminals; the cheapest tree is 0–1–2 of cost 2.
    let w = vec![0, 1, 1, 1, 0, 1, 1, 1, 0];
    let t = vec![0, 1, 2];
    assert_eq!(dp_steiner_tree::steiner_tree(3, &w, &t), 2);
    assert_eq!(brute_steiner(3, &w, &t), 2);
}

#[test]
fn test_star_with_expensive_leaf_edges() {
    // Centre 0 joined to 1, 2, 3 with weight 1; leaves joined to each other
    // with weight 9.  The best tree for terminals {1,2,3} is the star, cost 3.
    let w = vec![
        0, 1, 1, 1, //
        1, 0, 9, 9, //
        1, 9, 0, 9, //
        1, 9, 9, 0,
    ];
    let t = vec![1, 2, 3];
    assert_eq!(dp_steiner_tree::steiner_tree(4, &w, &t), 3);
    assert_eq!(brute_steiner(4, &w, &t), 3);
}

#[test]
fn test_zero_weights() {
    // Every edge is free.
    let n = 4;
    let w = vec![0usize; n * n];
    let t = vec![0, 1, 2, 3];
    assert_eq!(dp_steiner_tree::steiner_tree(n, &w, &t), 0);
    assert_eq!(brute_steiner(n, &w, &t), 0);
}

#[test]
fn test_duplicated_terminal() {
    // A repeated terminal must not change the answer.
    let w = vec![0, 1, 1, 0];
    let t = vec![0, 0];
    assert_eq!(dp_steiner_tree::steiner_tree(2, &w, &t), 0);
}

// ── Cross-check against the brute force ─────────────────────────────────────

/// Symmetric flat weight matrix on `n` vertices, `w[i][j] = i * 3 + j * 5 + 1`.
fn sym_weights(n: usize) -> Vec<usize> {
    let mut w = vec![0usize; n * n];
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            let mut x = i * 3 + j * 5 + 1;
            if i > j {
                x = j * 3 + i * 5 + 1;
            }
            w[i * n + j] = x;
            j += 1;
        }
        i += 1;
    }
    w
}

#[test]
fn test_agrees_with_brute_force_four_vertices() {
    let n = 4;
    let w = sym_weights(n);
    // Every subset of the 4 vertices as a terminal set.
    let mut mask: usize = 0;
    while mask < 16 {
        let mut t: Vec<usize> = Vec::new();
        let mut i: usize = 0;
        while i < n {
            let bit = 1usize << i;
            let has = mask & bit;
            if has != 0 {
                t.push(i);
            }
            i += 1;
        }
        let got = dp_steiner_tree::steiner_tree(n, &w, &t);
        let want = brute_steiner(n, &w, &t);
        assert_eq!(got, want);
        mask += 1;
    }
}

#[test]
fn test_agrees_with_brute_force_five_vertices() {
    let n = 5;
    let w = sym_weights(n);
    let mut mask: usize = 0;
    while mask < 32 {
        let mut t: Vec<usize> = Vec::new();
        let mut i: usize = 0;
        while i < n {
            let bit = 1usize << i;
            let has = mask & bit;
            if has != 0 {
                t.push(i);
            }
            i += 1;
        }
        let got = dp_steiner_tree::steiner_tree(n, &w, &t);
        let want = brute_steiner(n, &w, &t);
        assert_eq!(got, want);
        mask += 1;
    }
}

#[test]
fn test_agrees_with_brute_force_unit_weights() {
    // Unit weights: the answer is one less than the number of terminals when a
    // non-terminal can be reused as a hub, so this exercises the merge step.
    let n = 4;
    let mut w = vec![1usize; n * n];
    let mut i: usize = 0;
    while i < n {
        w[i * n + i] = 0;
        i += 1;
    }
    let t = vec![0, 1, 2, 3];
    assert_eq!(dp_steiner_tree::steiner_tree(n, &w, &t), 3);
    assert_eq!(brute_steiner(n, &w, &t), 3);
}
