// ── Independent brute force: enumerate every set of n-3 diagonals ───────────
//
// The brute force does NOT use the interval DP.  It enumerates all subsets of
// the candidate diagonals of size `n - 3`, keeps the ones that are pairwise
// non-crossing (i.e. all triangulations), and for each computes the cost
// directly from the induced triangular faces (a triple `a < b < c` is a face
// iff its three sides are each a polygon side or a chosen diagonal).  This is
// the same *declarative* cost the Lean spec uses, so an agreement between the
// brute force and the implementation checks exactly the DP recurrence against
// the specification's cost notion.

/// `true` iff vertices `a < b` form a polygon side (adjacent on the polygon).
fn is_diag(n: usize, a: usize, b: usize) -> bool {
    if a + 1 == b {
        false
    } else {
        if a == 0 {
            if b + 1 == n {
                false
            } else {
                true
            }
        } else {
            true
        }
    }
}

/// Weight of the edge `(i, j)` (`i < j`) in the flat row-major matrix.
fn edge_w(weight: &[usize], n: usize, i: usize, j: usize) -> usize {
    weight[i * n + j]
}

/// Number of set bits in `mask`.
fn bit_count(mask: usize) -> usize {
    let mut m = mask;
    let mut c: usize = 0;
    while m != 0 {
        let v = m / 2;
        let bit = m - v * 2;
        c += bit;
        m = v;
    }
    c
}

/// `true` iff diagonals `(a,b)` and `(c,d)` cross (endpoints interleave).
/// Assumes `a < b`, `c < d`, all endpoints distinct.
fn crosses(a: usize, b: usize, c: usize, d: usize) -> bool {
    if a < c {
        if c < b {
            if b < d {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        if a < d {
            if d < b {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// `true` iff the pair `(i, j)` (`i < j`) is a polygon side or one of the
/// `size` diagonals stored in `ca`/`cb`.
fn is_edge_or_diag(
    ca: &[usize],
    cb: &[usize],
    size: usize,
    n: usize,
    i: usize,
    j: usize,
) -> bool {
    let side = !is_diag(n, i, j);
    if side {
        true
    } else {
        let mut r = size; // sentinel: no diagonal matched
        let mut t = 0;
        while t < size {
            let a = ca[t];
            let b = cb[t];
            if a == i {
                if b == j {
                    if t < r {
                        r = t;
                    }
                }
            }
            t += 1;
        }
        if r != size {
            true
        } else {
            false
        }
    }
}

/// `true` iff the triple `a < b < c` is an induced face: all three of its sides
/// are a polygon side or a chosen diagonal.
fn triple_is_face(
    ca: &[usize],
    cb: &[usize],
    size: usize,
    n: usize,
    a: usize,
    b: usize,
    c: usize,
) -> bool {
    let ab = is_edge_or_diag(ca, cb, size, n, a, b);
    let bc = is_edge_or_diag(ca, cb, size, n, b, c);
    let ac = is_edge_or_diag(ca, cb, size, n, a, c);
    if ab {
        if bc {
            ac
        } else {
            false
        }
    } else {
        false
    }
}

/// Cost of the triangulation whose diagonals are `ca[0..size]`, `cb[0..size]`:
/// the sum over every induced face of its three edge weights.
fn set_cost(ca: &[usize], cb: &[usize], size: usize, n: usize, weight: &[usize]) -> usize {
    let mut total: usize = 0;
    let mut a = 0;
    while a < n {
        let mut b = a + 1;
        while b < n {
            let mut c = b + 1;
            while c < n {
                let face = triple_is_face(ca, cb, size, n, a, b, c);
                if face {
                    let w1 = edge_w(weight, n, a, b);
                    let w2 = edge_w(weight, n, b, c);
                    let w3 = edge_w(weight, n, a, c);
                    let s = w1 + w2 + w3;
                    let t2 = total + s;
                    total = t2;
                }
                c += 1;
            }
            b += 1;
        }
        a += 1;
    }
    total
}

/// Brute-force minimum-weight triangulation cost for the `n`-gon.
fn brute_triangulation(n: usize, weight: &[usize]) -> usize {
    if n < 3 {
        return 0;
    }
    // Candidate diagonals: pairs a < b that are not sides.
    let mut da = vec![0usize; n * n];
    let mut db = vec![0usize; n * n];
    let mut m = 0;
    let mut a = 0;
    while a < n {
        let mut b = a + 1;
        while b < n {
            let d = is_diag(n, a, b);
            if d {
                da[m] = a;
                db[m] = b;
                m += 1;
            }
            b += 1;
        }
        a += 1;
    }
    let need = n - 3;
    // 2^m, computed without shifts (shifts on a bare `usize` literal are brittle).
    let mut total_masks: usize = 1;
    let mut q = 0;
    while q < m {
        total_masks = total_masks * 2;
        q += 1;
    }
    let mut best: usize = 1000000000;
    let mut mask = 0;
    while mask < total_masks {
        let pc = bit_count(mask);
        if pc == need {
            let mut ca = vec![0usize; n];
            let mut cb = vec![0usize; n];
            let mut size = 0;
            let mut rem = mask;
            let mut t = 0;
            while t < m {
                let half = rem / 2;
                let bit = rem - half * 2;
                if bit == 1 {
                    ca[size] = da[t];
                    cb[size] = db[t];
                    size += 1;
                }
                rem = half;
                t += 1;
            }
            // Pairwise non-crossing?
            let mut ok = true;
            let mut x = 0;
            while x < size {
                let mut y = x + 1;
                while y < size {
                    let cr = crosses(ca[x], cb[x], ca[y], cb[y]);
                    if cr {
                        ok = false;
                    }
                    y += 1;
                }
                x += 1;
            }
            if ok {
                let cost = set_cost(&ca, &cb, size, n, weight);
                if cost < best {
                    best = cost;
                }
            }
        }
        mask += 1;
    }
    best
}

// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_triangle() {
    // A triangle needs no diagonal; its cost is the three side weights.
    let weight = vec![0, 1, 1, 0, 0, 1, 0, 0, 0];
    assert_eq!(dp_min_triangulation::min_triangulation(3, &weight), 3);
}

#[test]
fn test_quadrilateral() {
    // n = 4: sides all weight 1, diagonal (0,2) weight 2, diagonal (1,3) weight 5.
    // Cost with diagonal (0,2): 4 + 2*2 = 8;  with diagonal (1,3): 4 + 2*5 = 14.
    let mut weight = vec![0usize; 16];
    weight[0 * 4 + 1] = 1;
    weight[1 * 4 + 2] = 1;
    weight[2 * 4 + 3] = 1;
    weight[0 * 4 + 3] = 1;
    weight[0 * 4 + 2] = 2;
    weight[1 * 4 + 3] = 5;
    assert_eq!(dp_min_triangulation::min_triangulation(4, &weight), 8);
}

#[test]
fn test_all_ones_formula() {
    // With every edge weight 1, any triangulation has cost
    // (#sides) + 2*(#diagonals) = n + 2*(n-3) = 3n - 6.
    let mut n = 3;
    while n <= 8 {
        let weight = vec![1usize; n * n];
        let got = dp_min_triangulation::min_triangulation(n, &weight);
        assert_eq!(got, 3 * n - 6);
        n += 1;
    }
}

#[test]
fn test_degenerate_sizes() {
    // n < 3 has no triangle.
    let w0: Vec<usize> = vec![];
    assert_eq!(dp_min_triangulation::min_triangulation(0, &w0), 0);
    let w1 = vec![0usize];
    assert_eq!(dp_min_triangulation::min_triangulation(1, &w1), 0);
    let w2 = vec![0usize; 4];
    assert_eq!(dp_min_triangulation::min_triangulation(2, &w2), 0);
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_with_brute_force() {
    // A pseudo-random but fixed weight matrix (no external crates): the weight
    // at flat index `idx` is `(idx * idx + 7 * idx + 3) mod 13 + 1`.
    let mut n = 3;
    while n <= 7 {
        let sz = n * n;
        let mut weight = vec![0usize; sz];
        let mut idx = 0;
        while idx < sz {
            let t = idx * idx;
            let t2 = 7 * idx;
            let s = t + t2;
            let s2 = s + 3;
            let rem = s2 - (s2 / 13) * 13;
            let val = rem + 1;
            weight[idx] = val;
            idx += 1;
        }
        let got = dp_min_triangulation::min_triangulation(n, &weight);
        let want = brute_triangulation(n, &weight);
        assert_eq!(got, want);
        n += 1;
    }
}
