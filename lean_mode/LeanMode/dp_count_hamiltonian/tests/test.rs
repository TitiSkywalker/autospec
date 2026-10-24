// ── Independent brute force: enumerate every vertex sequence ────────────────

/// `x + y`.
fn t_add(x: usize, y: usize) -> usize {
    x + y
}

/// `x * y`.
fn t_mul(x: usize, y: usize) -> usize {
    x * y
}

/// `x mod m`, written without `%` (`UScalar.rem` is not always available in
/// translated test files).
fn t_rem(x: usize, m: usize) -> usize {
    let q = x / m;
    x - q * m
}

/// `b^e`.
fn t_ipow(b: usize, e: usize) -> usize {
    let mut acc = 1usize;
    let mut i = 0;
    while i < e {
        let t = t_mul(acc, b);
        acc = t;
        i += 1;
    }
    acc
}

/// Count the Hamiltonian paths by decoding every sequence in `[0, n)^n` and
/// keeping the ones that use every vertex exactly once and follow edges.
fn t_all_ham(n: usize, adj: &[usize]) -> usize {
    if n == 0 {
        return 1;
    }
    let total = t_ipow(n, n);
    let mut seq = vec![0usize; n];
    let mut count = 0usize;
    let mut code = 0usize;
    while code < total {
        // Decode `code` into a vertex sequence (base n).
        let mut c = code;
        let mut i = 0;
        while i < n {
            let d = t_rem(c, n);
            seq[i] = d;
            c = c / n;
            i += 1;
        }
        // Every vertex exactly once.
        let mut seen = vec![0usize; n];
        let mut i2 = 0;
        while i2 < n {
            let v = seq[i2];
            let t = t_add(seen[v], 1);
            seen[v] = t;
            i2 += 1;
        }
        let mut ok = 1usize;
        let mut i3 = 0;
        while i3 < n {
            let c2 = seen[i3];
            if c2 != 1 {
                ok = 0;
            }
            i3 += 1;
        }
        // Every consecutive pair is an edge.
        let mut k = 0;
        while k + 1 < n {
            let a1 = seq[k];
            let b1 = seq[k + 1];
            let idx = t_add(t_mul(a1, n), b1);
            let e = adj[idx];
            if e == 0 {
                ok = 0;
            }
            k += 1;
        }
        if ok == 1 {
            count += 1;
        }
        code += 1;
    }
    count
}

/// Adjacency matrix of the complete digraph: every entry off the diagonal.
fn t_complete(n: usize) -> Vec<usize> {
    let mut a = vec![0usize; n * n];
    let mut i = 0;
    while i < n {
        let mut j = 0;
        while j < n {
            if i != j {
                let idx = t_add(t_mul(i, n), j);
                a[idx] = 1;
            }
            j += 1;
        }
        i += 1;
    }
    a
}

// ── Degenerate inputs ───────────────────────────────────────────────────────

#[test]
fn test_no_vertices() {
    let a: Vec<usize> = vec![];
    assert_eq!(dp_count_hamiltonian::count_ham_paths(0, &a), 1);
}

#[test]
fn test_single_vertex() {
    let a = vec![0];
    assert_eq!(dp_count_hamiltonian::count_ham_paths(1, &a), 1);
    assert_eq!(t_all_ham(1, &a), 1);
}

// ── Known-answer tests ─────────────────────────────────────────────────────

#[test]
fn test_two_vertices_with_edge() {
    // 0 → 1 is the only path.
    let a = vec![0, 1, 0, 0];
    assert_eq!(dp_count_hamiltonian::count_ham_paths(2, &a), 1);
    assert_eq!(t_all_ham(2, &a), 1);
}

#[test]
fn test_two_vertices_both_edges() {
    // 0 → 1 and 1 → 0.
    let a = vec![0, 1, 1, 0];
    assert_eq!(dp_count_hamiltonian::count_ham_paths(2, &a), 2);
    assert_eq!(t_all_ham(2, &a), 2);
}

#[test]
fn test_two_vertices_no_edge() {
    let a = vec![0, 0, 0, 0];
    assert_eq!(dp_count_hamiltonian::count_ham_paths(2, &a), 0);
    assert_eq!(t_all_ham(2, &a), 0);
}

#[test]
fn test_complete_digraph() {
    // A complete digraph on n vertices has n! Hamiltonian paths.
    let mut n: usize = 1;
    while n <= 5 {
        let a = t_complete(n);
        let got = dp_count_hamiltonian::count_ham_paths(n, &a);
        let want = t_all_ham(n, &a);
        assert_eq!(got, want);
        n += 1;
    }
    // n! for n = 1..5.
    assert_eq!(dp_count_hamiltonian::count_ham_paths(5, &t_complete(5)), 120);
}

#[test]
fn test_tournament_has_paths() {
    // A transitive tournament: the only Hamiltonian path uses the order of the
    // total order, so exactly one path exists.
    let n = 4;
    let mut a = vec![0usize; n * n];
    let mut i = 0;
    while i < n {
        let mut j = 0;
        while j < n {
            if i < j {
                let idx = t_add(t_mul(i, n), j);
                a[idx] = 1;
            }
            j += 1;
        }
        i += 1;
    }
    assert_eq!(dp_count_hamiltonian::count_ham_paths(n, &a), 1);
    assert_eq!(t_all_ham(n, &a), 1);
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_with_brute_force_all_digraphs_three() {
    // Every one of the 2^6 = 64 digraphs on 3 vertices (6 ordered pairs).
    let n = 3;
    let pairs = 6;
    let mut code = 0usize;
    while code < (1usize << pairs) {
        let mut a = vec![0usize; n * n];
        let mut p = 0;
        while p < pairs {
            let bit = 1usize << p;
            let has = code & bit;
            if has != 0 {
                // Ordered pair number p of the 3 vertices: (0,1) (0,2) (1,0) ...
                let i = p / (n - 1);
                let r = t_rem(p, n - 1);
                let j = if r < i { r } else { r + 1 };
                let idx = t_add(t_mul(i, n), j);
                a[idx] = 1;
            }
            p += 1;
        }
        let got = dp_count_hamiltonian::count_ham_paths(n, &a);
        let want = t_all_ham(n, &a);
        assert_eq!(got, want);
        code += 1;
    }
}

#[test]
fn test_agrees_with_brute_force_sparse_four() {
    // Sparse pseudorandom digraphs on 4 vertices.
    let n = 4;
    let mut seed: usize = 1;
    let mut round = 0;
    while round < 24 {
        let mut a = vec![0usize; n * n];
        let mut i = 0;
        while i < n {
            let mut j = 0;
            while j < n {
                seed = t_rem(t_add(t_mul(seed, 7), 5), 1000);
                let idx = t_add(t_mul(i, n), j);
                if i != j {
                    let m = t_rem(seed, 3);
                    if m == 0 {
                        a[idx] = 1;
                    }
                }
                j += 1;
            }
            i += 1;
        }
        let got = dp_count_hamiltonian::count_ham_paths(n, &a);
        let want = t_all_ham(n, &a);
        assert_eq!(got, want);
        round += 1;
    }
}
