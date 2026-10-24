//! Tests for the matroid intersection testcase.
//!
//! Every test verifies four things about the program's output:
//!   (a) it is a 0/1 indicator,
//!   (b) the marked edges form a forest (graphic matroid independent),
//!   (c) no colour is used more than `k` times (partition matroid independent),
//!   (d) its size equals the brute-force maximum over all `2^m` subsets.

// ---------------------------------------------------------------------------
// Test-local helpers (names intentionally distinct from every `src/lib.rs` fn).
// ---------------------------------------------------------------------------

/// `a == b` as a `bool` value.
fn val_eq(a: usize, b: usize) -> bool {
    if a == b {
        true
    } else {
        false
    }
}

/// Boolean negation as a `bool` value.
fn bnot(b: bool) -> bool {
    if b {
        false
    } else {
        true
    }
}

/// Number of `1`s in a 0/1 vector.
fn count_ones(m: usize, sel: &[usize]) -> usize {
    let mut cnt: usize = 0;
    let mut i: usize = 0;
    while i < m {
        let s = sel[i] != 0;
        if s {
            cnt = cnt + 1;
        }
        i += 1;
    }
    cnt
}

/// Brute-force check: is the selected edge set acyclic (a forest) on `nv` nodes?
fn brute_is_forest(nv: usize, m: usize, from: &[usize], to: &[usize], sel: &[usize]) -> bool {
    let mut parent: Vec<usize> = vec![0usize; nv];
    let mut i: usize = 0;
    while i < nv {
        parent[i] = i;
        i += 1;
    }
    let mut forest: bool = true;
    let mut e: usize = 0;
    while e < m {
        let s = sel[e] != 0;
        if s {
            let u = from[e];
            let v = to[e];
            if u == v {
                forest = false;
            } else {
                let mut ru = u;
                while parent[ru] != ru {
                    ru = parent[ru];
                }
                let mut rv = v;
                while parent[rv] != rv {
                    rv = parent[rv];
                }
                if ru == rv {
                    forest = false;
                } else {
                    parent[ru] = rv;
                }
            }
        }
        e += 1;
    }
    forest
}

/// Brute-force check: does the selected edge set use each colour at most `k` times?
fn brute_colors_ok(m: usize, sel: &[usize], colors: &[usize], k: usize) -> bool {
    let mut ok: bool = true;
    let mut e: usize = 0;
    while e < m {
        let es = sel[e] != 0;
        if es {
            let c = colors[e];
            let mut cnt: usize = 0;
            let mut j: usize = 0;
            while j < m {
                let js = sel[j] != 0;
                let same = val_eq(colors[j], c);
                let inc = js && same;
                if inc {
                    cnt = cnt + 1;
                }
                j += 1;
            }
            if cnt > k {
                ok = false;
            }
        }
        e += 1;
    }
    ok
}

/// Brute-force maximum size of a common independent set over all `2^m` subsets.
fn brute_max_common(
    nv: usize,
    m: usize,
    from: &[usize],
    to: &[usize],
    colors: &[usize],
    k: usize,
) -> usize {
    let mut best: usize = 0;
    let mut total: usize = 1;
    let mut b: usize = 0;
    while b < m {
        total = total * 2;
        b += 1;
    }
    let mut mask: usize = 0;
    while mask < total {
        let mut sel: Vec<usize> = vec![0usize; m];
        let mut rem = mask;
        let mut i: usize = 0;
        while i < m {
            let half = rem / 2;
            let twice = half * 2;
            let bit = rem - twice;
            sel[i] = bit;
            rem = half;
            i += 1;
        }
        let forest = brute_is_forest(nv, m, from, to, &sel);
        let col_ok = brute_colors_ok(m, &sel, colors, k);
        let valid = forest && col_ok;
        if valid {
            let c = count_ones(m, &sel);
            if c > best {
                best = c;
            }
        }
        mask += 1;
    }
    best
}

/// Greedy baseline: scan the edges in order and add each one whenever the
/// resulting set is still common independent.  Suboptimal in general.
fn naive_greedy_size(
    nv: usize,
    m: usize,
    from: &[usize],
    to: &[usize],
    colors: &[usize],
    k: usize,
) -> usize {
    let mut sel: Vec<usize> = vec![0usize; m];
    let mut e: usize = 0;
    while e < m {
        sel[e] = 1;
        let forest = brute_is_forest(nv, m, from, to, &sel);
        let col_ok = brute_colors_ok(m, &sel, colors, k);
        let feasible = forest && col_ok;
        let revert = bnot(feasible);
        if revert {
            sel[e] = 0;
        }
        e += 1;
    }
    count_ones(m, &sel)
}

/// Run the algorithm, validate (a)-(d), and return the size.
fn check_result(
    nv: usize,
    m: usize,
    from: &[usize],
    to: &[usize],
    colors: &[usize],
    k: usize,
) -> usize {
    let got = graph_matroid_intersection::matroid_intersection(nv, m, from, to, colors, k);
    let mut i: usize = 0;
    while i < m {
        assert!(got[i] <= 1); // (a) 0/1
        i += 1;
    }
    let forest = brute_is_forest(nv, m, from, to, &got);
    assert!(forest); // (b)
    let col_ok = brute_colors_ok(m, &got, colors, k);
    assert!(col_ok); // (c)
    let cnt = count_ones(m, &got);
    let best = brute_max_common(nv, m, from, to, colors, k);
    assert_eq!(cnt, best); // (d)
    cnt
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// The exchange-graph augmentation is *necessary* here: naive greedy is
/// suboptimal.  Ground set `{(0,1)@c1, (1,2)@c1, (0,1)@c2}` with `k = 1`.
/// Greedy takes edge 0, then can add nothing (colour clash with edge 1, cycle
/// with edge 2) and returns size 1.  The only size-2 common independent set is
/// `{edge 1, edge 2}`, reached only by the augmenting path
/// `edge1 -> edge0 -> edge2` (flip: drop edge0, add edge1 and edge2).
#[test]
fn test_exchange_needed() {
    let from = [0usize, 1, 0];
    let to = [1usize, 2, 1];
    let colors = [1usize, 1, 2];
    let size = check_result(3, 3, &from, &to, &colors, 1);
    assert_eq!(size, 2);

    let got = graph_matroid_intersection::matroid_intersection(3, 3, &from, &to, &colors, 1);
    assert_eq!(got[0], 0);
    assert_eq!(got[1], 1);
    assert_eq!(got[2], 1);

    // And the greedy baseline really is suboptimal on this instance.
    let greedy = naive_greedy_size(3, 3, &from, &to, &colors, 1);
    assert_eq!(greedy, 1);
}

/// A path (already a forest) with distinct colours: everything is selected.
#[test]
fn test_forest_all_selected() {
    let from = [0usize, 1, 2, 3];
    let to = [1usize, 2, 3, 4];
    let colors = [0usize, 1, 2, 3];
    let size = check_result(5, 4, &from, &to, &colors, 1);
    assert_eq!(size, 4);
}

/// Same star, same colour: capacity `k` caps the answer exactly at `k`.
#[test]
fn test_star_capacity() {
    let from = [0usize, 0, 0];
    let to = [1usize, 2, 3];
    let colors = [5usize, 5, 5];
    assert_eq!(check_result(4, 3, &from, &to, &colors, 1), 1);
    assert_eq!(check_result(4, 3, &from, &to, &colors, 2), 2);
    assert_eq!(check_result(4, 3, &from, &to, &colors, 3), 3);
}

/// Triangle with distinct colours: a rainbow forest has size `nv - 1 = 2`.
#[test]
fn test_triangle_distinct_colors() {
    let from = [0usize, 1, 0];
    let to = [1usize, 2, 2];
    let colors = [0usize, 1, 2];
    let size = check_result(3, 3, &from, &to, &colors, 1);
    assert_eq!(size, 2);
}

/// `K4` with six distinct colours: a spanning tree (3 edges) is achievable.
#[test]
fn test_k4_distinct_colors() {
    let from = [0usize, 0, 0, 1, 1, 2];
    let to = [1usize, 2, 3, 2, 3, 3];
    let colors = [0usize, 1, 2, 3, 4, 5];
    let size = check_result(4, 6, &from, &to, &colors, 1);
    assert_eq!(size, 3);
}

/// Disconnected: a parallel pair in one component, a single edge in another.
#[test]
fn test_disconnected() {
    let from = [0usize, 0, 2];
    let to = [1usize, 1, 3];
    let colors = [0usize, 1, 0];
    let size = check_result(4, 3, &from, &to, &colors, 1);
    assert_eq!(size, 2);
}

/// A 4-cycle with a repeated colour: only three colours, so `k = 1` caps at 3.
#[test]
fn test_repeated_colors() {
    let from = [0usize, 1, 2, 3, 0];
    let to = [1usize, 2, 3, 0, 2];
    let colors = [1usize, 1, 2, 2, 3];
    let size = check_result(4, 5, &from, &to, &colors, 1);
    assert_eq!(size, 3);
}

/// Empty ground set.
#[test]
fn test_m_zero() {
    let from: [usize; 0] = [];
    let to: [usize; 0] = [];
    let colors: [usize; 0] = [];
    let got = graph_matroid_intersection::matroid_intersection(0, 0, &from, &to, &colors, 1);
    assert_eq!(got.len(), 0);
}

/// Regression for the exchange-digraph source/sink convention: a self-loop
/// (or any cycle-closing edge) must never be pulled into the support.
///
/// `nv = 3, m = 4, k = 1`: `e0 = (0,0)` is a self-loop, `e1 = (1,2)`,
/// `e2 = (1,0)`, `e3 = (1,2)`.  The buggy program returned `[1,0,1,1]`,
/// selecting the self-loop `e0` and reaching size 3 while the true maximum is
/// 2 (`e0` is never acyclic; `e1`/`e3` are parallel).  This test pins down both
/// acyclicity (via `check_result`'s forest check) and the optimum.
#[test]
fn test_regression_self_loop_not_selected() {
    let from = [0usize, 1, 1, 1];
    let to = [0usize, 2, 0, 2];
    let colors = [0usize, 2, 2, 1];
    let size = check_result(3, 4, &from, &to, &colors, 1);
    assert_eq!(size, 2);

    let got = graph_matroid_intersection::matroid_intersection(3, 4, &from, &to, &colors, 1);
    assert_eq!(got[0], 0); // the self-loop e0 must not be selected
}

/// Regression for the exchange-digraph source/sink convention: the augmenting
/// path must not terminate at a colour-only sink, which would overflow a
/// colour class.
///
/// `nv = 6, m = 7, k = 2`, `colors = [3,0,3,0,3,0,0]`.  The buggy program
/// returned `[0,1,1,1,0,1,0]`, selecting `e1,e2,e3,e5` whose colours are
/// `0,3,0,0` — colour `0` used three times with `k = 2`.  The colour-capacity
/// and maximum checks in `check_result` pin this down (true optimum is 4).
#[test]
fn test_regression_colour_capacity() {
    let from = [5usize, 2, 5, 2, 3, 4, 1];
    let to = [4usize, 1, 4, 3, 2, 3, 2];
    let colors = [3usize, 0, 3, 0, 3, 0, 0];
    let size = check_result(6, 7, &from, &to, &colors, 2);
    assert_eq!(size, 4);
}
