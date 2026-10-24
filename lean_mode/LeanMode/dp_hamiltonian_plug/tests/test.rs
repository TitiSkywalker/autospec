// ── Independent brute force: enumerate grid Hamiltonian cycles ──────────────
//
// Cells are indexed row-major as `row * w + col`.  Fix cell `0` as the start
// of a walk, DFS over simple walks that return to `0`.  Each undirected cycle
// is discovered twice (once per direction), so the total is halved.

/// Are cells `a` and `b` orthogonally adjacent in a `w`-column grid?
fn gadj(a: usize, b: usize, w: usize) -> bool {
    let ra = a / w;
    let ca = a - ra * w;
    let rb = b / w;
    let cb = b - rb * w;
    let dr = if ra > rb { ra - rb } else { rb - ra };
    let dc = if ca > cb { ca - cb } else { cb - ca };
    let s = dr + dc;
    if s == 1 { true } else { false }
}

/// Count simple walks from the start through all cells and back to `0`.
fn bf_dfs(vis: &mut Vec<usize>, w: usize, cur: usize, cnt: usize, n: usize) -> usize {
    if cnt == n {
        if gadj(cur, 0, w) {
            return 1;
        } else {
            return 0;
        }
    }
    let mut total: usize = 0;
    let mut nxt: usize = 0;
    while nxt < n {
        if vis[nxt] == 0 {
            if gadj(cur, nxt, w) {
                vis[nxt] = 1;
                let s = bf_dfs(vis, w, nxt, cnt + 1, n);
                total += s;
                vis[nxt] = 0;
            }
        }
        nxt += 1;
    }
    total
}

/// Brute-force number of Hamiltonian cycles of the `w × h` grid.
fn bf_ham_cycles(w: usize, h: usize) -> usize {
    let n = w * h;
    if n == 0 {
        return 0;
    }
    let mut vis = vec![0usize; n];
    vis[0] = 1;
    let t = bf_dfs(&mut vis, w, 0, 1, n);
    t / 2
}

// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_2x2() {
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(2, 2), 1);
}

#[test]
fn test_2xn() {
    // The only Hamiltonian cycle of a `2 × n` grid is the perimeter.
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(2, 3), 1);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(2, 4), 1);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(2, 5), 1);
}

#[test]
fn test_3x4() {
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(3, 4), 2);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(4, 3), 2);
}

#[test]
fn test_4x4() {
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(4, 4), 6);
}

#[test]
fn test_3x6() {
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(3, 6), 4);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn test_1xn_zero() {
    // A path graph has no cycle.
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(1, 1), 0);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(1, 4), 0);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(4, 1), 0);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(0, 3), 0);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(3, 0), 0);
}

#[test]
fn test_odd_zero() {
    // A grid is bipartite; an odd number of vertices admits no cycle.
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(3, 3), 0);
    assert_eq!(dp_hamiltonian_plug::ham_cycles_grid(5, 5), 0);
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_brute() {
    let mut w = 1;
    while w <= 4 {
        let mut h = 1;
        while h <= 4 {
            let got = dp_hamiltonian_plug::ham_cycles_grid(w, h);
            let want = bf_ham_cycles(w, h);
            assert_eq!(got, want);
            h += 1;
        }
        w += 1;
    }
}
