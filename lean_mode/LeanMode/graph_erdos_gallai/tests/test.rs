// ── Known-answer tests for `is_graphical` (Havel–Hakimi) ────────────────────

#[test]
fn test_triangle() {
    // [2,2,2] — the triangle K3.
    let d = vec![2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_odd_sum() {
    // [1,1,1] — odd degree sum: not graphical.
    let d = vec![1, 1, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), false);
}

#[test]
fn test_k4() {
    // [3,3,3,3] — the complete graph K4.
    let d = vec![3, 3, 3, 3];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_three_threes() {
    // [3,3,3] — odd degree sum (9): not graphical.
    let d = vec![3, 3, 3];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), false);
}

#[test]
fn test_single_zero() {
    // [0] — one isolated vertex.
    let d = vec![0];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_empty() {
    // [] — the empty graph.
    assert_eq!(graph_erdos_gallai::is_graphical(&[]), true);
}

#[test]
fn test_two_edges() {
    // [1,1,1,1] — a perfect matching (2K2), or the 4-cycle.
    let d = vec![1, 1, 1, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_cycle_c5() {
    // [2,2,2,2,2] — the 5-cycle C5.
    let d = vec![2, 2, 2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_k5() {
    // [4,4,4,4,4] — the complete graph K5.
    let d = vec![4, 4, 4, 4, 4];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_degree_too_large() {
    // [5,5,5,5] — a vertex of degree 5 needs 5 distinct neighbours,
    // but there are only 4 vertices.
    let d = vec![5, 5, 5, 5];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), false);
}

#[test]
fn test_unsorted_one_edge() {
    // [1,0,1] — one edge plus an isolated vertex (input is not sorted).
    let d = vec![1, 0, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_split_five() {
    // [3,3,2,2,2] — a split graph on 5 vertices.
    let d = vec![3, 3, 2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_long_graphical() {
    // [11,9,9,7,7,5,5,4,3,3,2,1] on 12 vertices (degree sum 66 = 2*33).
    let d = vec![11, 9, 9, 7, 7, 5, 5, 4, 3, 3, 2, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), true);
}

#[test]
fn test_long_not_graphical() {
    // [10,9,8,8,6,5,2,1,1,0,0] — even degree sum (50) but not graphical.
    let d = vec![10, 9, 8, 8, 6, 5, 2, 1, 1, 0, 0];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), false);
}

#[test]
fn test_long_not_graphical_reduction() {
    // [8,6,6,6,3,1,1,1,0,0] — the reduction runs out of neighbours.
    let d = vec![8, 6, 6, 6, 3, 1, 1, 1, 0, 0];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), false);
}

#[test]
fn test_cross_check_brute_force() {
    // Exhaustive check against all 2^(n choose 2) simple graphs.
    let a = vec![2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&a), brute_graphical(&a));
    let b = vec![1, 1, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&b), brute_graphical(&b));
    let c = vec![3, 3, 3, 3];
    assert_eq!(graph_erdos_gallai::is_graphical(&c), brute_graphical(&c));
    let e = vec![1, 1, 1, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&e), brute_graphical(&e));
    let f = vec![2, 2, 2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&f), brute_graphical(&f));
    let g = vec![3, 3, 2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&g), brute_graphical(&g));
    let h = vec![5, 5, 5, 5];
    assert_eq!(graph_erdos_gallai::is_graphical(&h), brute_graphical(&h));
    let k = vec![1, 0, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&k), brute_graphical(&k));
    let m = vec![2, 4, 4, 2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&m), brute_graphical(&m));
    let p = vec![3, 3, 3, 1, 1, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&p), brute_graphical(&p));
}

#[test]
fn test_cross_check_erdos_gallai() {
    // Compare with the independent Erdős–Gallai inequalities (the input is
    // non-increasing, as required there).
    let a = vec![2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&a), eg_reference(&a));
    let b = vec![1, 1, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&b), eg_reference(&b));
    let c = vec![4, 4, 4, 4, 4];
    assert_eq!(graph_erdos_gallai::is_graphical(&c), eg_reference(&c));
    let d = vec![5, 5, 5, 5];
    assert_eq!(graph_erdos_gallai::is_graphical(&d), eg_reference(&d));
    let e = vec![3, 3, 2, 2, 2];
    assert_eq!(graph_erdos_gallai::is_graphical(&e), eg_reference(&e));
    let f = vec![11, 9, 9, 7, 7, 5, 5, 4, 3, 3, 2, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&f), eg_reference(&f));
    let g = vec![10, 9, 8, 8, 6, 5, 2, 1, 1, 0, 0];
    assert_eq!(graph_erdos_gallai::is_graphical(&g), eg_reference(&g));
    let h = vec![8, 6, 6, 6, 3, 1, 1, 1, 0, 0];
    assert_eq!(graph_erdos_gallai::is_graphical(&h), eg_reference(&h));
    let k = vec![9, 8, 8, 7, 7, 6, 5, 4, 2, 1, 1];
    assert_eq!(graph_erdos_gallai::is_graphical(&k), eg_reference(&k));
    let m = vec![6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6];
    assert_eq!(graph_erdos_gallai::is_graphical(&m), eg_reference(&m));
}

// ── Test-local reference helpers ────────────────────────────────────────────
// (Distinct names from anything in `src/lib.rs`.)

/// Exhaustive brute force: is there a simple graph on `n = d.len()` vertices
/// whose degree sequence (as a multiset) equals `d`?  Bit `b` of `mask` says
/// whether the `b`-th unordered vertex pair is an edge, so the enumeration
/// runs over all `2^(n choose 2)` graphs.  Only usable for small `n`.
fn brute_graphical(d: &[usize]) -> bool {
    let n = d.len();

    // Number of unordered vertex pairs.
    let mut pairs: usize = 0;
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = i + 1;
        while j < n {
            pairs += 1;
            j += 1;
        }
        i += 1;
    }

    // 2 ^ pairs
    let mut total: usize = 1;
    let mut c: usize = 0;
    while c < pairs {
        total = total * 2;
        c += 1;
    }

    let mut found: bool = false;
    let mut mask: usize = 0;
    while mask < total {
        // Degrees of the graph encoded by `mask`.
        let mut deg: Vec<usize> = vec![0usize; n];
        let mut rem = mask;
        let mut u: usize = 0;
        while u < n {
            let mut v: usize = u + 1;
            while v < n {
                let bit = rem % 2;
                if bit == 1 {
                    let du = deg[u] + 1;
                    deg[u] = du;
                    let dv = deg[v] + 1;
                    deg[v] = dv;
                }
                rem = rem / 2;
                v += 1;
            }
            u += 1;
        }

        // Multiset comparison of `deg` and `d`.
        let mut same: bool = true;
        let mut x: usize = 0;
        while x < n {
            let val = d[x];
            let mut ca: usize = 0;
            let mut y: usize = 0;
            while y < n {
                if d[y] == val {
                    ca += 1;
                }
                y += 1;
            }
            let mut cb: usize = 0;
            let mut z: usize = 0;
            while z < n {
                if deg[z] == val {
                    cb += 1;
                }
                z += 1;
            }
            if ca != cb {
                same = false;
            }
            x += 1;
        }
        if same {
            found = true;
        }

        mask += 1;
    }

    found
}

/// The Erdős–Gallai inequalities for a non-increasing sequence `d`:
/// the sum is even and for every `k`,
/// `sum_{i<k} d[i] <= k*(k-1) + sum_{i>=k} min(d[i], k)`.
fn eg_reference(d: &[usize]) -> bool {
    let n = d.len();

    let mut sum: usize = 0;
    let mut i: usize = 0;
    while i < n {
        sum += d[i];
        i += 1;
    }

    let mut ok: bool = sum % 2 != 1;
    let mut k: usize = 1;
    while k <= n {
        let mut left: usize = 0;
        let mut a: usize = 0;
        while a < k {
            left += d[a];
            a += 1;
        }
        let mut right: usize = k * (k - 1);
        let mut b: usize = k;
        while b < n {
            let dd = d[b];
            let m = if dd < k { dd } else { k };
            right += m;
            b += 1;
        }
        if left > right {
            ok = false;
        }
        k += 1;
    }

    ok
}
