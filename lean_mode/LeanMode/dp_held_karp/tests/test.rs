// ── Independent brute force: enumerate every Hamiltonian cycle ───────────────

/// Shortest Hamiltonian-cycle cost by exhaustive search: enumerate the order in
/// which the vertices `1 .. n` are visited, with `0` fixed as the start.
fn brute_ham(n: usize, dist: &[usize]) -> usize {
    if n == 0 {
        return 0;
    }
    let mut used = vec![0usize; n];
    used[0] = 1;
    brute_walk(0, n, dist, &mut used)
}

/// Minimum cost of a walk starting at `v` that visits every vertex still unmarked
/// in `used` exactly once and then returns to vertex `0`.
fn brute_walk(v: usize, n: usize, dist: &[usize], used: &mut Vec<usize>) -> usize {
    let mut remaining: usize = 0;
    let mut i = 0;
    while i < n {
        if used[i] == 0 {
            remaining += 1;
        }
        i += 1;
    }
    if remaining == 0 {
        // Every vertex has been visited: return to the start.
        return dist[v * n];
    }
    let mut cands: Vec<usize> = Vec::new();
    let mut u = 0;
    while u < n {
        if used[u] == 0 {
            used[u] = 1;
            let sub = brute_walk(u, n, dist, used);
            used[u] = 0;
            let edge = dist[v * n + u];
            cands.push(edge + sub);
        }
        u += 1;
    }
    let mut best = cands[0];
    let mut k = 1;
    while k < cands.len() {
        let c = cands[k];
        if c < best {
            best = c;
        }
        k += 1;
    }
    best
}

// ── Degenerate inputs ───────────────────────────────────────────────────────

#[test]
fn test_no_vertices() {
    let d: Vec<usize> = vec![];
    assert_eq!(dp_held_karp::held_karp(0, &d), 0);
    assert_eq!(brute_ham(0, &d), 0);
}

#[test]
fn test_single_vertex() {
    // One vertex: the only tour is the self-loop 0 → 0.
    let d = vec![7];
    assert_eq!(dp_held_karp::held_karp(1, &d), 7);
    assert_eq!(brute_ham(1, &d), 7);
}

// ── Known-answer tests ─────────────────────────────────────────────────────

#[test]
fn test_two_vertices() {
    // 0 → 1 → 0 costs d(0,1) + d(1,0) = 1 + 1 = 2.
    let d = vec![0, 1, 1, 0];
    assert_eq!(dp_held_karp::held_karp(2, &d), 2);
}

#[test]
fn test_asymmetric_three_vertices() {
    // d(0,1)=1, d(1,2)=1, d(2,0)=1 → tour 0→1→2→0 costs 3.
    let d = vec![0, 1, 2, 2, 0, 1, 1, 2, 0];
    assert_eq!(dp_held_karp::held_karp(3, &d), 3);
    assert_eq!(brute_ham(3, &d), 3);
}

#[test]
fn test_asymmetric_four_vertices() {
    // Row-major distances:
    //   [0, 3, 8, 4]
    //   [2, 0, 1, 9]
    //   [7, 6, 0, 5]
    //   [1, 4, 3, 0]
    // Best tour: 0 → 1 → 2 → 3 → 0 = 3 + 1 + 5 + 1 = 10.
    let d = vec![0, 3, 8, 4, 2, 0, 1, 9, 7, 6, 0, 5, 1, 4, 3, 0];
    assert_eq!(dp_held_karp::held_karp(4, &d), 10);
    assert_eq!(brute_ham(4, &d), 10);
}

#[test]
fn test_zero_distances() {
    // All distances zero: every tour is free.
    let n = 4;
    let d = vec![0usize; n * n];
    assert_eq!(dp_held_karp::held_karp(n, &d), 0);
    assert_eq!(brute_ham(n, &d), 0);
}

#[test]
fn test_symmetric_line() {
    // Four cities on a line, d(i,j) = |i - j|.
    // Best tour: 0 → 1 → 2 → 3 → 0 = 1 + 1 + 1 + 3 = 6.
    let d = vec![0, 1, 2, 3, 1, 0, 1, 2, 2, 1, 0, 1, 3, 2, 1, 0];
    assert_eq!(dp_held_karp::held_karp(4, &d), 6);
    assert_eq!(brute_ham(4, &d), 6);
}

#[test]
fn test_large_diagonal_is_ignored() {
    // The diagonal (self-loops) is never used by a tour, so a huge diagonal must
    // not change the answer.
    let mut d = vec![0, 1, 1, 0];
    d[0] = 1000;
    d[3] = 1000;
    assert_eq!(dp_held_karp::held_karp(2, &d), 2);
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_with_brute_force() {
    let mut n: usize = 1;
    while n <= 5 {
        let mut d = vec![0usize; n * n];
        let mut i: usize = 0;
        while i < n {
            let mut j: usize = 0;
            while j < n {
                let idx = i * n + j;
                let w = i * 3 + j * 5 + 1;
                d[idx] = w;
                j += 1;
            }
            i += 1;
        }
        let got = dp_held_karp::held_karp(n, &d);
        let want = brute_ham(n, &d);
        assert_eq!(got, want);
        n += 1;
    }
}

#[test]
fn test_agrees_with_brute_force_small_values() {
    // Same cross-check with tiny, heavily repeated distances (many ties, which
    // forces the DP to break ties consistently with the brute force).
    let mut n: usize = 1;
    while n <= 5 {
        let mut d = vec![0usize; n * n];
        let mut i: usize = 0;
        while i < n {
            let mut j: usize = 0;
            while j < n {
                let idx = i * n + j;
                let w = i + j;
                if i == j {
                    d[idx] = 0;
                } else {
                    d[idx] = w;
                }
                j += 1;
            }
            i += 1;
        }
        let got = dp_held_karp::held_karp(n, &d);
        let want = brute_ham(n, &d);
        assert_eq!(got, want);
        n += 1;
    }
}
