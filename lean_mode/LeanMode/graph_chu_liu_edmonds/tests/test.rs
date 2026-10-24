//! Tests for the Chu–Liu / Edmonds minimum arborescence.
//!
//! The interesting cases are the ones where the greedy "minimum in-edge"
//! choice closes a cycle and the cycle has to be contracted before the true
//! optimum is reached.  Everything is cross-checked against a brute-force
//! enumerator over all in-edge assignments (test-local helpers only, none of
//! them shadows a name from `src/lib.rs`).

// ---------------------------------------------------------------------------
// brute force: enumerate every assignment of one in-edge to each non-root node
// ---------------------------------------------------------------------------

/// Is the parent assignment `chosen` (`chosen[v]` = edge index of the in-edge of
/// `v`, `m` meaning "none") a spanning arborescence rooted at `r`?
fn brute_valid(
    nv: usize,
    from: &[usize],
    to: &[usize],
    chosen: &[usize],
    r: usize,
    m: usize,
) -> bool {
    let mut ok: bool = true;

    // every non-root node has a genuine in-edge (no self-loop, in range)
    let mut v: usize = 0;
    while v < nv {
        if v != r {
            let ei = chosen[v];
            if ei >= m {
                ok = false;
            } else {
                let hd = to[ei];
                if hd != v {
                    ok = false;
                }
                let tl = from[ei];
                if tl == v {
                    ok = false;
                }
                if tl >= nv {
                    ok = false;
                }
            }
        }
        v += 1;
    }

    // the chosen edges are pairwise distinct
    let mut a: usize = 0;
    while a < nv {
        if a != r {
            let mut b: usize = a + 1;
            while b < nv {
                if b != r {
                    if chosen[a] == chosen[b] {
                        ok = false;
                    }
                }
                b += 1;
            }
        }
        a += 1;
    }

    // every non-root node reaches the root by following parent pointers
    if ok {
        let mut a2: usize = 0;
        while a2 < nv {
            if a2 != r {
                let mut x = a2;
                let mut step: usize = 0;
                while step < nv {
                    if x != r {
                        let ei = chosen[x];
                        let px = from[ei];
                        x = px;
                    }
                    step += 1;
                }
                if x != r {
                    ok = false;
                }
            }
            a2 += 1;
        }
    }

    ok
}

/// Exact brute force: minimum weight of an arborescence rooted at `r`, or
/// `(false, 0)` when none exists.  Enumerates all `m^(nv-1)` in-edge
/// assignments (with `m = from.len()`).
fn brute_optimum(nv: usize, from: &[usize], to: &[usize], w: &[usize], r: usize) -> (bool, usize) {
    let m = from.len();

    // the list of non-root nodes
    let mut nodes: Vec<usize> = Vec::new();
    let mut v: usize = 0;
    while v < nv {
        if v != r {
            nodes.push(v);
        }
        v += 1;
    }
    let nk = nodes.len();

    let mut combos: usize = 1;
    let mut i: usize = 0;
    while i < nk {
        combos = combos * m;
        i += 1;
    }

    let mut chosen: Vec<usize> = vec![0usize; nv];
    let mut best: usize = 0;
    let mut found: bool = false;
    let mut c: usize = 0;
    while c < combos {
        // decode `c` in base `m` into the assignment
        let mut cc = c;
        let mut k: usize = 0;
        while k < nk {
            let rest = cc / m;
            let digit = cc % m;
            cc = rest;
            let node = nodes[k];
            chosen[node] = digit;
            k += 1;
        }

        let ok = brute_valid(nv, from, to, &chosen, r, m);
        if ok {
            let mut sum: usize = 0;
            let mut v2: usize = 0;
            while v2 < nv {
                if v2 != r {
                    let ei = chosen[v2];
                    let wi = w[ei];
                    sum = sum + wi;
                }
                v2 += 1;
            }
            if !found {
                best = sum;
                found = true;
            } else {
                if sum < best {
                    best = sum;
                }
            }
        }
        c += 1;
    }

    let bw = if found { best } else { 0usize };
    (found, bw)
}

/// Compare the algorithm with the brute force on one instance.
fn brute_cross_check(nv: usize, from: &[usize], to: &[usize], w: &[usize], r: usize) {
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(nv, from, to, w, r);
    let (bx, bw) = brute_optimum(nv, from, to, w, r);
    assert_eq!(ex, bx);
    assert_eq!(wt, bw);
}

// ---------------------------------------------------------------------------
// hand-checked instances
// ---------------------------------------------------------------------------

#[test]
fn test_dag_no_contraction() {
    // 0 -> 1 (2), 1 -> 2 (3), 0 -> 2 (10): a DAG, minimum in-edges already form
    // the optimum arborescence 0->1, 1->2 of weight 5.
    let from = vec![0, 1, 0];
    let to = vec![1, 2, 2];
    let w = vec![2, 3, 10];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(3, &from, &to, &w, 0);
    assert!(ex);
    assert_eq!(wt, 5);
}

#[test]
fn test_two_cycle_contraction() {
    // 0 (root) -> 1 (10), 1 -> 2 (1), 2 -> 1 (1).
    // The greedy choice gives 2->1 and 1->2, a 2-cycle that must be contracted;
    // the optimum is 0->1, 1->2 with weight 11.
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 1];
    let w = vec![10, 1, 1];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(3, &from, &to, &w, 0);
    assert!(ex);
    assert_eq!(wt, 11);
}

#[test]
fn test_three_cycle_contraction() {
    // 1 -> 2 (1), 2 -> 3 (1), 3 -> 1 (1), 0 (root) -> 1 (5).
    // Greedy picks the 3-cycle; contracting it leaves the single edge
    // 0 -> {1,2,3} of re-weighted cost 4, total 1+1+1+4 = 7.
    let from = vec![0, 1, 2, 3];
    let to = vec![1, 2, 3, 1];
    let w = vec![5, 1, 1, 1];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(4, &from, &to, &w, 0);
    assert!(ex);
    assert_eq!(wt, 7);
}

#[test]
fn test_nested_cycles_contraction() {
    // Two rounds of contraction:
    //   0 (root) -> 1 (100)
    //   1 -> 2 (1), 2 -> 1 (1)          (2-cycle, cheap)
    //   2 -> 3 (1), 3 -> 2 (1)          (another 2-cycle)
    // Greedy: node1 <- 2 (1), node2 <- 1 (1) or 3 (1), node3 <- 2 (1).
    // After contraction the optimum is 0->1 plus a tree inside {1,2,3}.
    let from = vec![0, 1, 2, 2, 3];
    let to = vec![1, 2, 1, 3, 2];
    let w = vec![100, 1, 1, 1, 1];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(4, &from, &to, &w, 0);
    assert!(ex);
    // must use 0->1 (weight 100), then 1->2 and 2->3 (weight 1 each)
    assert_eq!(wt, 102);
}

#[test]
fn test_parallel_edges() {
    // parallel edges 0->1 with weights 5 and 2, plus 0->2 (7) and 1->2 (3).
    let from = vec![0, 0, 0, 1];
    let to = vec![1, 1, 2, 2];
    let w = vec![5, 2, 7, 3];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(3, &from, &to, &w, 0);
    assert!(ex);
    assert_eq!(wt, 5);
}

#[test]
fn test_parallel_edges_tie_prefers_any() {
    // two parallel edges 0->1 of equal weight 4
    let from = vec![0, 0];
    let to = vec![1, 1];
    let w = vec![4, 4];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(2, &from, &to, &w, 0);
    assert!(ex);
    assert_eq!(wt, 4);
}

#[test]
fn test_unreachable_node() {
    // node 2 has no incoming edge: no arborescence.
    let from = vec![0];
    let to = vec![1];
    let w = vec![1];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(3, &from, &to, &w, 0);
    assert!(!ex);
    assert_eq!(wt, 0);
}

#[test]
fn test_single_node() {
    // empty arborescence, weight 0
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(1, &[], &[], &[], 0);
    assert!(ex);
    assert_eq!(wt, 0);
}

#[test]
fn test_self_loops_only() {
    // node 1 is only reachable through its own self-loop: no arborescence.
    let from = vec![0, 1];
    let to = vec![0, 1];
    let w = vec![1, 1];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(2, &from, &to, &w, 0);
    assert!(!ex);
    assert_eq!(wt, 0);
}

#[test]
fn test_self_loop_plus_real_edge() {
    // the cheap self-loop 1->1 (2) must be ignored; 1 is spanned by 0->1 (9).
    let from = vec![0, 0, 1];
    let to = vec![0, 1, 1];
    let w = vec![1, 9, 2];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(2, &from, &to, &w, 0);
    assert!(ex);
    assert_eq!(wt, 9);
}

#[test]
fn test_root_not_zero() {
    // root 2: 2->0 (4), 0->1 (1), 1->0 (1)
    let from = vec![2, 0, 1];
    let to = vec![0, 1, 0];
    let w = vec![4, 1, 1];
    let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(3, &from, &to, &w, 2);
    assert!(ex);
    assert_eq!(wt, 5);
}

// ---------------------------------------------------------------------------
// brute-force cross-checks
// ---------------------------------------------------------------------------

#[test]
fn test_brute_two_cycle() {
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 1];
    let w = vec![10, 1, 1];
    brute_cross_check(3, &from, &to, &w, 0);
}

#[test]
fn test_brute_three_cycle() {
    let from = vec![0, 1, 2, 3];
    let to = vec![1, 2, 3, 1];
    let w = vec![5, 1, 1, 1];
    brute_cross_check(4, &from, &to, &w, 0);
}

#[test]
fn test_brute_contraction_beats_greedy() {
    // The unique optimum needs the contraction step: a cheap 2-cycle that has
    // to be broken at a loss elsewhere.
    //   0 -> 1 (10), 1 -> 2 (1), 2 -> 3 (1), 3 -> 2 (1), 0 -> 3 (6)
    let from = vec![0, 1, 2, 3, 0];
    let to = vec![1, 2, 3, 2, 3];
    let w = vec![10, 1, 1, 1, 6];
    brute_cross_check(4, &from, &to, &w, 0);
}

#[test]
fn test_brute_parallel_and_self_loop() {
    let from = vec![0, 0, 0, 1, 1, 2];
    let to = vec![1, 1, 2, 1, 2, 2];
    let w = vec![7, 3, 9, 2, 4, 5];
    brute_cross_check(3, &from, &to, &w, 0);
}

#[test]
fn test_brute_unreachable() {
    let from = vec![0, 0, 2];
    let to = vec![1, 2, 0];
    let w = vec![1, 1, 1];
    brute_cross_check(3, &from, &to, &w, 0);
}

#[test]
fn test_brute_four_nodes_dense() {
    // 4 vertices, 9 edges, several cycles
    let from = vec![0, 0, 1, 1, 2, 2, 3, 3, 1];
    let to = vec![1, 2, 2, 3, 1, 3, 1, 2, 0];
    let w = vec![3, 8, 5, 2, 4, 6, 7, 1, 9];
    brute_cross_check(4, &from, &to, &w, 0);
}

#[test]
fn test_brute_root_two_of_four() {
    let from = vec![0, 1, 2, 2, 3, 0];
    let to = vec![1, 2, 0, 3, 2, 3];
    let w = vec![2, 2, 2, 5, 1, 4];
    brute_cross_check(4, &from, &to, &w, 2);
}

#[test]
fn test_brute_single_node() {
    brute_cross_check(1, &[], &[], &[], 0);
}

#[test]
fn test_brute_two_nodes_parallel() {
    let from = vec![0, 0, 0];
    let to = vec![1, 1, 1];
    let w = vec![5, 5, 2];
    brute_cross_check(2, &from, &to, &w, 0);
}

// ---------------------------------------------------------------------------
// deterministic pseudorandom cross-check (LCG, no external crates)
// ---------------------------------------------------------------------------

#[test]
fn test_random_cross_check() {
    let mut seed: usize = 12345;
    let mut trial: usize = 0;
    while trial < 40 {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        let nv = 2 + seed % 4; // 2..5
        seed = (seed * 1103515245 + 12345) % 2147483648;
        let m = seed % 9;
        seed = (seed * 1103515245 + 12345) % 2147483648;
        let r = seed % nv;
        let mut from: Vec<usize> = Vec::new();
        let mut to: Vec<usize> = Vec::new();
        let mut w: Vec<usize> = Vec::new();
        let mut i: usize = 0;
        while i < m {
            seed = (seed * 1103515245 + 12345) % 2147483648;
            let a = seed % nv;
            seed = (seed * 1103515245 + 12345) % 2147483648;
            let b = seed % nv;
            seed = (seed * 1103515245 + 12345) % 2147483648;
            let c = 1 + seed % 7;
            from.push(a);
            to.push(b);
            w.push(c);
            i += 1;
        }
        let (ex, wt) = graph_chu_liu_edmonds::optimum_branching(nv, &from, &to, &w, r);
        let (bx, bw) = brute_optimum(nv, &from, &to, &w, r);
        assert_eq!(ex, bx);
        assert_eq!(wt, bw);
        trial += 1;
    }
}
