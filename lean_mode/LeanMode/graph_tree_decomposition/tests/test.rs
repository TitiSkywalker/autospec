// Small, fast tests for `graph_tree_decomposition`.  Everything here is
// translated into `Test_test.lean` and `#eval`'d, so the sizes stay tiny and
// every indexed access is kept inside a helper (Lean's `Vec` is `ListN`-backed,
// so every `v[i]` is O(len)).  The heavy randomized differential tests live in
// the untranslated `#[cfg(test)] mod tests` of `src/lib.rs`.

use graph_tree_decomposition::min_elimination_width;
use graph_tree_decomposition::tree_decomposition;
use graph_tree_decomposition::TreeDecomposition;

// Flat row-major index helper (distinct from the private `idx` in lib.rs, and
// not a prefix of any lib function name).
fn flat(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

// Start offset of bag `i` in `bag_data` (sum of the preceding lengths).
// All arithmetic goes through two-parameter helpers so that Aeneas keeps the
// operands `usize`-typed.
fn bag_addr(start: usize, t: usize) -> usize {
    let r = start + t;
    r
}

fn bag_start(td: &TreeDecomposition, i: usize) -> usize {
    let mut s: usize = 0;
    let mut k: usize = 0;
    while k < i {
        s += td.bag_len[k];
        k += 1;
    }
    s
}

// Does bag `i` contain vertex `v`?
fn bag_has(td: &TreeDecomposition, i: usize, v: usize) -> bool {
    let s = bag_start(td, i);
    let l = td.bag_len[i];
    let mut t: usize = 0;
    let mut found: usize = 0;
    while t < l {
        let a = bag_addr(s, t);
        let x = td.bag_data[a];
        if x == v {
            found = 1;
        }
        t += 1;
    }
    if found != 0 {
        true
    } else {
        false
    }
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
        adj[p] = 1;
        let q = flat(b, n, a);
        adj[q] = 1;
        e += 1;
    }
    adj
}

// The number of violations of the tree-decomposition properties; 0 iff all of
// them hold.  Checks: CSR well-formedness, non-empty duplicate-free bags with
// in-range vertices, the bag-forest shape (parent strictly larger or the
// sentinel `n`, every bag reaching the sentinel), the cover property, and the
// running-intersection property in the form "every non-head member `v` of bag
// `i` also sits in the parent bag of `i`" (which forces the bags containing
// `v` to form a connected upward chain from bag `i` to the bag of `v`).
fn td_violations(n: usize, adj: &[usize], pi: &[usize], td: &TreeDecomposition) -> usize {
    let mut bad: usize = 0;

    // total = sum of bag_len; must equal bag_data.len()
    let mut total: usize = 0;
    let mut i: usize = 0;
    while i < n {
        total += td.bag_len[i];
        i += 1;
    }
    if total != td.bag_data.len() {
        bad += 1;
    }

    // bags: non-empty, entries < n, no internal duplicate
    let mut i2: usize = 0;
    while i2 < n {
        let l = td.bag_len[i2];
        if l == 0 {
            bad += 1;
        }
        let s = bag_start(td, i2);
        let mut a: usize = 0;
        while a < l {
            let aa = bag_addr(s, a);
            let va = td.bag_data[aa];
            if va >= n {
                bad += 1;
            }
            let mut b: usize = a + 1;
            while b < l {
                let ab = bag_addr(s, b);
                let vb = td.bag_data[ab];
                if va == vb {
                    bad += 1;
                }
                b += 1;
            }
            a += 1;
        }
        i2 += 1;
    }

    // parent array: in range, strictly increasing, reaching the sentinel
    let mut i3: usize = 0;
    while i3 < n {
        let p = td.parent[i3];
        if p > n {
            bad += 1;
        }
        if p < n {
            if p <= i3 {
                bad += 1;
            }
        }
        let mut cur = i3;
        let mut steps: usize = 0;
        while steps < n + 1 {
            if cur < n {
                cur = td.parent[cur];
            }
            steps += 1;
        }
        if cur != n {
            bad += 1;
        }
        i3 += 1;
    }

    // cover: every edge lies in some bag
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = u + 1;
        while v < n {
            let e = adj[flat(u, n, v)];
            if e != 0 {
                let mut hits: usize = 0;
                let mut k: usize = 0;
                while k < n {
                    if bag_has(td, k, u) {
                        if bag_has(td, k, v) {
                            hits += 1;
                        }
                    }
                    k += 1;
                }
                if hits == 0 {
                    bad += 1;
                }
            }
            v += 1;
        }
        u += 1;
    }

    // running intersection: every non-head member of bag `i` is also in the
    // parent bag of `i`
    let mut i4: usize = 0;
    while i4 < n {
        let l4 = td.bag_len[i4];
        let s4 = bag_start(td, i4);
        let head = pi[i4];
        let p4 = td.parent[i4];
        let mut t4: usize = 0;
        while t4 < l4 {
            let a4 = bag_addr(s4, t4);
            let v4 = td.bag_data[a4];
            if v4 != head {
                if p4 < n {
                    if bag_has(td, p4, v4) {
                    } else {
                        bad += 1;
                    }
                } else {
                    bad += 1;
                }
            }
            t4 += 1;
        }
        i4 += 1;
    }

    bad
}

#[test]
fn test_path3() {
    // path 0-1-2
    let n = 3;
    let adj = mk_adj(n, &[0, 1], &[1, 2]);
    let pi: Vec<usize> = vec![0, 1, 2];
    let td = tree_decomposition(n, &adj, &pi);
    assert_eq!(td.bag_len[0], 2);
    assert_eq!(td.bag_len[1], 2);
    assert_eq!(td.bag_len[2], 1);
    assert_eq!(td.bag_data[0], 0);
    assert_eq!(td.bag_data[1], 1);
    assert_eq!(td.bag_data[2], 1);
    assert_eq!(td.bag_data[3], 2);
    assert_eq!(td.bag_data[4], 2);
    assert_eq!(td.parent[0], 1);
    assert_eq!(td.parent[1], 2);
    assert_eq!(td.parent[2], 3);
    assert_eq!(td_violations(n, &adj, &pi, &td), 0);
}

#[test]
fn test_triangle() {
    // triangle 0-1-2-0, width 2
    let n = 3;
    let adj = mk_adj(n, &[0, 1, 2], &[1, 2, 0]);
    let pi: Vec<usize> = vec![0, 1, 2];
    let td = tree_decomposition(n, &adj, &pi);
    assert_eq!(td.bag_len[0], 3);
    assert_eq!(td.bag_len[1], 2);
    assert_eq!(td.bag_len[2], 1);
    assert_eq!(td.parent[0], 1);
    assert_eq!(td.parent[1], 2);
    assert_eq!(td.parent[2], 3);
    assert_eq!(td.bag_data[0], 0);
    assert_eq!(td.bag_data[1], 1);
    assert_eq!(td.bag_data[2], 2);
    assert_eq!(td_violations(n, &adj, &pi, &td), 0);
}

#[test]
fn test_cycle4_needs_fill() {
    // 4-cycle 0-1-2-3-0 in the order (0,1,2,3): the elimination game adds the
    // fill edge 1-3, so bag 1 = {1,2,3} and vertex 3 lands in bags 0, 1, 2, 3.
    let n = 4;
    let adj = mk_adj(n, &[0, 1, 2, 3], &[1, 2, 3, 0]);
    let pi: Vec<usize> = vec![0, 1, 2, 3];
    let td = tree_decomposition(n, &adj, &pi);
    assert_eq!(td.bag_len[0], 3);
    assert_eq!(td.bag_len[1], 3);
    assert_eq!(td.bag_len[2], 2);
    assert_eq!(td.bag_len[3], 1);
    assert_eq!(td.bag_data[0], 0);
    assert_eq!(td.bag_data[1], 1);
    assert_eq!(td.bag_data[2], 3);
    assert_eq!(td.bag_data[3], 1);
    assert_eq!(td.bag_data[4], 2);
    assert_eq!(td.bag_data[5], 3);
    assert_eq!(td.parent[0], 1);
    assert_eq!(td.parent[1], 2);
    assert_eq!(td.parent[2], 3);
    assert_eq!(td.parent[3], 4);
    assert_eq!(td_violations(n, &adj, &pi, &td), 0);
}

#[test]
fn test_k4_reverse_order() {
    // K4 in the order (3,2,1,0): a clique tree, width 3 = tw(K4)
    let n = 4;
    let adj = mk_adj(n, &[0, 0, 0, 1, 1, 2], &[1, 2, 3, 2, 3, 3]);
    let pi: Vec<usize> = vec![3, 2, 1, 0];
    let td = tree_decomposition(n, &adj, &pi);
    assert_eq!(td.bag_len[0], 4);
    assert_eq!(td.bag_len[1], 3);
    assert_eq!(td.bag_len[2], 2);
    assert_eq!(td.bag_len[3], 1);
    assert_eq!(td.bag_data[0], 3);
    assert_eq!(td.bag_data[1], 2);
    assert_eq!(td.bag_data[2], 1);
    assert_eq!(td.bag_data[3], 0);
    assert_eq!(td.parent[0], 1);
    assert_eq!(td.parent[1], 2);
    assert_eq!(td.parent[2], 3);
    assert_eq!(td.parent[3], 4);
    assert_eq!(td_violations(n, &adj, &pi, &td), 0);
}

#[test]
fn test_disconnected_two_edges() {
    // edges {0,1} and {2,3}: two connected components, so the bag forest has
    // two roots (bag 1 and bag 3), and the two roots are NOT edges of a tree.
    let n = 4;
    let adj = mk_adj(n, &[0, 2], &[1, 3]);
    let pi: Vec<usize> = vec![0, 1, 2, 3];
    let td = tree_decomposition(n, &adj, &pi);
    assert_eq!(td.bag_len[0], 2);
    assert_eq!(td.bag_len[1], 1);
    assert_eq!(td.bag_len[2], 2);
    assert_eq!(td.bag_len[3], 1);
    assert_eq!(td.parent[0], 1);
    assert_eq!(td.parent[1], 4);
    assert_eq!(td.parent[2], 3);
    assert_eq!(td.parent[3], 4);
    assert_eq!(td_violations(n, &adj, &pi, &td), 0);
}

#[test]
fn test_min_width_small() {
    // Brute-force treewidth: K3 -> 2, C4 -> 2, path 0-1-2-3 -> 1.
    let n3 = 3;
    let tri = mk_adj(n3, &[0, 1, 2], &[1, 2, 0]);
    assert_eq!(min_elimination_width(n3, &tri), 2);
    let n4 = 4;
    let c4 = mk_adj(n4, &[0, 1, 2, 3], &[1, 2, 3, 0]);
    assert_eq!(min_elimination_width(n4, &c4), 2);
    let p4 = mk_adj(n4, &[0, 1, 2], &[1, 2, 3]);
    assert_eq!(min_elimination_width(n4, &p4), 1);
}
