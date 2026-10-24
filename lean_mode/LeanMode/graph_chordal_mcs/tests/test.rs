// Flat row-major index helper (distinct from the private `idx` in `src/lib.rs`).
fn flat(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

// Build a flat, row-major, symmetric 0/1 adjacency matrix from an undirected
// edge list given as parallel `from` / `to` slices.
fn mk_adj(n: usize, from: &[usize], to: &[usize]) -> Vec<usize> {
    let mut adj: Vec<usize> = vec![0usize; n * n];
    let m = from.len();
    let mut e: usize = 0;
    while e < m {
        let a = from[e];
        let b = to[e];
        let p = flat(a, n, b);
        let q = flat(b, n, a);
        adj[p] = 1;
        adj[q] = 1;
        e += 1;
    }
    adj
}

// `pi` is a permutation of `0..n` (length n, all entries < n, no repeats).
fn check_perm(pi: &[usize], n: usize) -> bool {
    let mut seen: Vec<usize> = vec![0usize; n];
    let mut bad: usize = 0;
    let mut i: usize = 0;
    while i < n {
        let v = pi[i];
        if v < n {
            seen[v] = 1;
        } else {
            bad = 1;
        }
        i += 1;
    }
    let mut s: usize = 0;
    let mut j: usize = 0;
    while j < n {
        s += seen[j];
        j += 1;
    }
    if bad == 0 {
        if s == n {
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[test]
fn test_p4_path() {
    // path 0-1-2-3 (chordal: a tree)
    let adj = mk_adj(4, &[0, 1, 2], &[1, 2, 3]);
    let pi = graph_chordal_mcs::chordal_mcs(4, &adj);
    assert_eq!(pi.len(), 4);
    assert_eq!(pi[0], 3);
    assert_eq!(pi[1], 2);
    assert_eq!(pi[2], 1);
    assert_eq!(pi[3], 0);
}

#[test]
fn test_triangle() {
    // triangle 0-1-2-0
    let adj = mk_adj(3, &[0, 1, 2], &[1, 2, 0]);
    let pi = graph_chordal_mcs::chordal_mcs(3, &adj);
    assert_eq!(pi.len(), 3);
    assert_eq!(pi[0], 2);
    assert_eq!(pi[1], 1);
    assert_eq!(pi[2], 0);
}

#[test]
fn test_k4() {
    // complete graph on 4 vertices
    let adj = mk_adj(4, &[0, 0, 0, 1, 1, 2], &[1, 2, 3, 2, 3, 3]);
    let pi = graph_chordal_mcs::chordal_mcs(4, &adj);
    assert_eq!(pi.len(), 4);
    assert_eq!(pi[0], 3);
    assert_eq!(pi[1], 2);
    assert_eq!(pi[2], 1);
    assert_eq!(pi[3], 0);
    // structural fact: for K4 the MCS order is a permutation of 0..4
    assert!(check_perm(&pi, 4));
}

#[test]
fn test_c4_not_chordal() {
    // 4-cycle 0-1-2-3-0 (not chordal)
    let adj = mk_adj(4, &[0, 1, 2, 3], &[1, 2, 3, 0]);
    let pi = graph_chordal_mcs::chordal_mcs(4, &adj);
    assert_eq!(pi.len(), 4);
    assert_eq!(pi[0], 3);
    assert_eq!(pi[1], 2);
    assert_eq!(pi[2], 1);
    assert_eq!(pi[3], 0);
}

#[test]
fn test_c5_not_chordal() {
    // 5-cycle 0-1-2-3-4-0 (not chordal)
    let adj = mk_adj(5, &[0, 1, 2, 3, 4], &[1, 2, 3, 4, 0]);
    let pi = graph_chordal_mcs::chordal_mcs(5, &adj);
    assert_eq!(pi.len(), 5);
    assert_eq!(pi[0], 4);
    assert_eq!(pi[1], 3);
    assert_eq!(pi[2], 2);
    assert_eq!(pi[3], 1);
    assert_eq!(pi[4], 0);
}

#[test]
fn test_diamond_chordal_not_complete() {
    // two triangles {0,1,2} and {0,1,3} sharing the edge {0,1}: K4 minus {2,3}
    let adj = mk_adj(4, &[0, 0, 0, 1, 1], &[1, 2, 3, 2, 3]);
    let pi = graph_chordal_mcs::chordal_mcs(4, &adj);
    assert_eq!(pi.len(), 4);
    assert_eq!(pi[0], 3);
    assert_eq!(pi[1], 2);
    assert_eq!(pi[2], 1);
    assert_eq!(pi[3], 0);
    assert!(check_perm(&pi, 4));
}

#[test]
fn test_2tree() {
    // 2-tree: triangle {0,1,2}, then 3 glued onto {0,1}, then 4 onto {1,2}
    let adj = mk_adj(5, &[0, 1, 2, 0, 1, 1, 2], &[1, 2, 0, 3, 3, 4, 4]);
    let pi = graph_chordal_mcs::chordal_mcs(5, &adj);
    assert_eq!(pi.len(), 5);
    assert_eq!(pi[0], 4);
    assert_eq!(pi[1], 3);
    assert_eq!(pi[2], 2);
    assert_eq!(pi[3], 1);
    assert_eq!(pi[4], 0);
}

#[test]
fn test_disconnected() {
    // edge {0,1}, triangle {2,3,4}, isolated vertex 5
    let adj = mk_adj(6, &[0, 2, 3, 4], &[1, 3, 4, 2]);
    let pi = graph_chordal_mcs::chordal_mcs(6, &adj);
    assert_eq!(pi.len(), 6);
    assert_eq!(pi[0], 5);
    assert_eq!(pi[1], 4);
    assert_eq!(pi[2], 3);
    assert_eq!(pi[3], 2);
    assert_eq!(pi[4], 1);
    assert_eq!(pi[5], 0);
    assert!(check_perm(&pi, 6));
}

#[test]
fn test_star_center_5() {
    // star K_{1,5} centred at vertex 5 (chordal: a tree); selection jumps to
    // the high-index centre on step 2, so pi is not monotone.
    let adj = mk_adj(6, &[5, 5, 5, 5, 5], &[0, 1, 2, 3, 4]);
    let pi = graph_chordal_mcs::chordal_mcs(6, &adj);
    assert_eq!(pi.len(), 6);
    assert_eq!(pi[0], 4);
    assert_eq!(pi[1], 3);
    assert_eq!(pi[2], 2);
    assert_eq!(pi[3], 1);
    assert_eq!(pi[4], 5);
    assert_eq!(pi[5], 0);
    assert!(check_perm(&pi, 6));
}

#[test]
fn test_star_center_3() {
    // star centred at vertex 3 with a distinguished leaf 0 (chordal tree)
    let adj = mk_adj(4, &[0, 3, 3], &[3, 1, 2]);
    let pi = graph_chordal_mcs::chordal_mcs(4, &adj);
    assert_eq!(pi.len(), 4);
    assert_eq!(pi[0], 2);
    assert_eq!(pi[1], 1);
    assert_eq!(pi[2], 3);
    assert_eq!(pi[3], 0);
    assert!(check_perm(&pi, 4));
}
