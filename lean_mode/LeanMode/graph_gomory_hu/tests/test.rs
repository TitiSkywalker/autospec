//! Tests for the Gomory–Hu cut tree.
//!
//! Besides the small hand-checked examples, every test runs a brute-force
//! cross-check: for every pair `u ≠ v` it enumerates *all* vertex subsets `S`
//! with `u ∈ S`, `v ∉ S`, takes the minimum cut capacity, and compares it with
//! the minimum tree-edge weight on the `u`–`v` path of the returned tree.

// ---------------------------------------------------------------------------
// brute-force reference (test-local; the names are deliberately disjoint from
// every function in `src/lib.rs`)
// ---------------------------------------------------------------------------

/// Flat index for a row-major `n × n` matrix.
fn bf_idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Capacity of the cut induced by the indicator vector `inx`
/// (edges from the `inx == 1` side to the `inx == 0` side).
fn bf_cut_capacity(cap: &[usize], n: usize, inx: &[usize]) -> usize {
    let mut total: usize = 0;
    let mut a: usize = 0;
    while a < n {
        let mut b: usize = 0;
        while b < n {
            if inx[a] == 1 {
                if inx[b] == 0 {
                    let w = cap[bf_idx(a, n, b)];
                    let t = total + w;
                    total = t;
                }
            }
            b += 1;
        }
        a += 1;
    }
    total
}

/// Brute-force minimum `u`–`v` cut: minimum of `bf_cut_capacity` over all
/// subsets separating `u` from `v` (enumerated as a binary counter, without
/// shifts or divisions).
fn bf_mincut(cap: &[usize], n: usize, u: usize, v: usize) -> usize {
    let mut inx: Vec<usize> = vec![0usize; n];
    let mut best: usize = 0;
    let mut best_set: bool = false;
    let mut done: bool = false;
    while !done {
        if inx[u] == 1 {
            if inx[v] == 0 {
                let c = bf_cut_capacity(cap, n, &inx);
                if !best_set {
                    best = c;
                    best_set = true;
                } else {
                    if c < best {
                        best = c;
                    }
                }
            }
        }
        // Advance the binary counter (low bit first).
        let mut carry: bool = true;
        let mut i: usize = 0;
        while i < n {
            if carry {
                if inx[i] == 0 {
                    inx[i] = 1;
                    carry = false;
                } else {
                    inx[i] = 0;
                }
            }
            i += 1;
        }
        if carry {
            done = true;
        }
    }
    best
}

/// Depth of `x` in the forest given by `parent` (walking `parent` at most `n`
/// steps; correct whenever the chain from `x` reaches the root).
fn bf_depth(n: usize, parent: &[usize], x: usize) -> usize {
    let mut d: usize = 0;
    let mut y = x;
    let mut i: usize = 0;
    while i < n {
        if y != 0 {
            y = parent[y];
            d += 1;
        }
        i += 1;
    }
    d
}

/// Minimum tree-edge `weight` on the `u`–`v` path of the tree `parent` /
/// `weight` (align the two depths, then climb together, collecting the edge
/// `weight[x]` of every vertex `x` left behind).
fn bf_path_min(n: usize, parent: &[usize], weight: &[usize], u: usize, v: usize) -> usize {
    let mut edge_w: Vec<usize> = Vec::new();
    let mut ua = u;
    let mut va = v;
    let mut du = bf_depth(n, parent, u);
    let mut dv = bf_depth(n, parent, v);
    let mut k: usize = 0;
    while k < n {
        if du > dv {
            edge_w.push(weight[ua]);
            ua = parent[ua];
            du -= 1;
        } else {
            if dv > du {
                edge_w.push(weight[va]);
                va = parent[va];
                dv -= 1;
            } else {
                if ua != va {
                    edge_w.push(weight[ua]);
                    edge_w.push(weight[va]);
                    ua = parent[ua];
                    va = parent[va];
                }
            }
        }
        k += 1;
    }
    let mut res: usize = 0;
    let mut i: usize = 0;
    while i < edge_w.len() {
        let w = edge_w[i];
        if i == 0 {
            res = w;
        } else {
            if w < res {
                res = w;
            }
        }
        i += 1;
    }
    res
}

/// `parent` really is a tree rooted at the (implicit) vertex `0`.
fn bf_tree_ok(n: usize, parent: &[usize]) -> bool {
    let mut ok: bool = true;
    if parent[0] != 0 {
        ok = false;
    }
    let mut i: usize = 0;
    while i < n {
        let mut x = parent[i];
        let mut steps: usize = 0;
        while steps < n {
            if x != 0 {
                x = parent[x];
            }
            steps += 1;
        }
        if x != 0 {
            ok = false;
        }
        i += 1;
    }
    ok
}

/// Element-wise equality of two slices.
fn bf_same(a: &[usize], b: &[usize]) -> bool {
    let mut ok: bool = true;
    let la = a.len();
    let lb = b.len();
    if la != lb {
        ok = false;
    } else {
        let mut i: usize = 0;
        while i < la {
            let x = a[i];
            let y = b[i];
            if x != y {
                ok = false;
            }
            i += 1;
        }
    }
    ok
}

/// The full Gomory–Hu property: for every `u ≠ v`, the brute-force minimum
/// `u`–`v` cut equals the minimum tree-edge weight on the `u`–`v` tree path.
fn bf_cross_check(n: usize, cap: &[usize], parent: &[usize], weight: &[usize]) -> bool {
    let mut ok: bool = true;
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = 0;
        while v < n {
            if u != v {
                let a = bf_mincut(cap, n, u, v);
                let b = bf_path_min(n, parent, weight, u, v);
                if a != b {
                    ok = false;
                }
            }
            v += 1;
        }
        u += 1;
    }
    ok
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[test]
fn test_six_vertex_example() {
    // The 6-vertex example graph: the 6-cycle 0-1-2-3-4-5-0 with capacities
    // 1, 7, 1, 3, 3, 1 plus the chord 1-5 of capacity 2.
    let cap: Vec<usize> = vec![
        0, 1, 0, 0, 0, 1,
        1, 0, 7, 0, 0, 2,
        0, 7, 0, 1, 0, 0,
        0, 0, 1, 0, 3, 0,
        0, 0, 0, 3, 0, 3,
        1, 2, 0, 0, 3, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(6, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 1, 1, 1, 1];
    let want_weight: Vec<usize> = vec![0, 2, 8, 4, 4, 4];
    assert!(bf_tree_ok(6, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(6, &cap, &parent, &weight));
}

#[test]
fn test_cycle4_unit() {
    // C4 with unit capacities: every pair has min cut 2.
    let cap: Vec<usize> = vec![
        0, 1, 0, 1,
        1, 0, 1, 0,
        0, 1, 0, 1,
        1, 0, 1, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(4, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0, 0];
    let want_weight: Vec<usize> = vec![0, 2, 2, 2];
    assert!(bf_tree_ok(4, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(4, &cap, &parent, &weight));
}

#[test]
fn test_path() {
    // The path 0-1-2-3 with unit capacities: every pair has min cut 1, so the
    // tree is the path itself with all weights 1.
    let cap: Vec<usize> = vec![
        0, 1, 0, 0,
        1, 0, 1, 0,
        0, 1, 0, 1,
        0, 0, 1, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(4, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 1, 2];
    let want_weight: Vec<usize> = vec![0, 1, 1, 1];
    assert!(bf_tree_ok(4, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(4, &cap, &parent, &weight));
}

#[test]
fn test_k4_unit() {
    // K4 with unit capacities: every pair has min cut 3.
    let cap: Vec<usize> = vec![
        0, 1, 1, 1,
        1, 0, 1, 1,
        1, 1, 0, 1,
        1, 1, 1, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(4, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0, 0];
    let want_weight: Vec<usize> = vec![0, 3, 3, 3];
    assert!(bf_tree_ok(4, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(4, &cap, &parent, &weight));
}

#[test]
fn test_bridge() {
    // Two triangles {0,1,2} and {3,4,5} (unit capacities) joined by the bridge
    // 2-3 of capacity 5.  The tree has to put the bridge weight 5 on the 2-3
    // path and 2 on every other path.
    let cap: Vec<usize> = vec![
        0, 1, 1, 0, 0, 0,
        1, 0, 1, 0, 0, 0,
        1, 1, 0, 5, 0, 0,
        0, 0, 5, 0, 1, 1,
        0, 0, 0, 1, 0, 1,
        0, 0, 0, 1, 1, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(6, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0, 2, 3, 3];
    let want_weight: Vec<usize> = vec![0, 2, 2, 5, 2, 2];
    assert!(bf_tree_ok(6, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(6, &cap, &parent, &weight));
}

#[test]
fn test_two_triangles() {
    // Two triangles {0,1,2} and {3,4,5} joined by the single bridge 2-3 of
    // capacity 2: every cut separating the two triangles has value 2.
    let cap: Vec<usize> = vec![
        0, 1, 1, 0, 0, 0,
        1, 0, 1, 0, 0, 0,
        1, 1, 0, 2, 0, 0,
        0, 0, 2, 0, 1, 1,
        0, 0, 0, 1, 0, 1,
        0, 0, 0, 1, 1, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(6, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0, 2, 3, 3];
    let want_weight: Vec<usize> = vec![0, 2, 2, 2, 2, 2];
    assert!(bf_tree_ok(6, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(6, &cap, &parent, &weight));
}

#[test]
fn test_single_vertex() {
    let cap: Vec<usize> = vec![0];
    let (parent, weight) = graph_gomory_hu::gomory_hu(1, &cap);
    let want_parent: Vec<usize> = vec![0];
    let want_weight: Vec<usize> = vec![0];
    assert!(bf_tree_ok(1, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(1, &cap, &parent, &weight));
}

#[test]
fn test_two_vertices() {
    // A single edge of capacity 7: min cut 7.
    let cap: Vec<usize> = vec![0, 7, 7, 0];
    let (parent, weight) = graph_gomory_hu::gomory_hu(2, &cap);
    let want_parent: Vec<usize> = vec![0, 0];
    let want_weight: Vec<usize> = vec![0, 7];
    assert!(bf_tree_ok(2, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(2, &cap, &parent, &weight));
}

#[test]
fn test_disconnected_zero_capacities() {
    // Edges 0-1 (3) and 2-3 (4) only: the graph has two components, so the min
    // cut between them is 0 and the tree must contain a weight-0 edge.
    let cap: Vec<usize> = vec![
        0, 3, 0, 0,
        3, 0, 0, 0,
        0, 0, 0, 4,
        0, 0, 4, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(4, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0, 2];
    let want_weight: Vec<usize> = vec![0, 3, 0, 4];
    assert!(bf_tree_ok(4, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(4, &cap, &parent, &weight));
}

#[test]
fn test_no_edges() {
    // No edges at all: every min cut is 0.
    let cap: Vec<usize> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let (parent, weight) = graph_gomory_hu::gomory_hu(3, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0];
    let want_weight: Vec<usize> = vec![0, 0, 0];
    assert!(bf_tree_ok(3, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(3, &cap, &parent, &weight));
}

#[test]
fn test_weighted_five() {
    let cap: Vec<usize> = vec![
        0, 2, 3, 0, 0,
        2, 0, 1, 4, 0,
        3, 1, 0, 5, 6,
        0, 4, 5, 0, 2,
        0, 0, 6, 2, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(5, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 1, 2, 2];
    let want_weight: Vec<usize> = vec![0, 5, 7, 10, 8];
    assert!(bf_tree_ok(5, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(5, &cap, &parent, &weight));
}

#[test]
fn test_seven_cycle() {
    // C7 with unit capacities: every pair has min cut 2.
    let cap: Vec<usize> = vec![
        0, 1, 0, 0, 0, 0, 1,
        1, 0, 1, 0, 0, 0, 0,
        0, 1, 0, 1, 0, 0, 0,
        0, 0, 1, 0, 1, 0, 0,
        0, 0, 0, 1, 0, 1, 0,
        0, 0, 0, 0, 1, 0, 1,
        1, 0, 0, 0, 0, 1, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(7, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0, 0, 0, 0, 0];
    let want_weight: Vec<usize> = vec![0, 2, 2, 2, 2, 2, 2];
    assert!(bf_tree_ok(7, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(7, &cap, &parent, &weight));
}

#[test]
fn test_eight_vertices() {
    // Two K4s {0,1,2,3} and {4,5,6,7} (unit capacities) joined by the single
    // bridge 3-4 of capacity 2: pairs inside a K4 have min cut 3, pairs across
    // the bridge have min cut 2.
    let cap: Vec<usize> = vec![
        0, 1, 1, 1, 0, 0, 0, 0,
        1, 0, 1, 1, 0, 0, 0, 0,
        1, 1, 0, 1, 0, 0, 0, 0,
        1, 1, 1, 0, 2, 0, 0, 0,
        0, 0, 0, 2, 0, 1, 1, 1,
        0, 0, 0, 0, 1, 0, 1, 1,
        0, 0, 0, 0, 1, 1, 0, 1,
        0, 0, 0, 0, 1, 1, 1, 0,
    ];
    let (parent, weight) = graph_gomory_hu::gomory_hu(8, &cap);
    let want_parent: Vec<usize> = vec![0, 0, 0, 0, 3, 4, 4, 4];
    let want_weight: Vec<usize> = vec![0, 3, 3, 3, 2, 3, 3, 3];
    assert!(bf_tree_ok(8, &parent));
    assert!(bf_same(&parent, &want_parent));
    assert!(bf_same(&weight, &want_weight));
    assert!(bf_cross_check(8, &cap, &parent, &weight));
}
