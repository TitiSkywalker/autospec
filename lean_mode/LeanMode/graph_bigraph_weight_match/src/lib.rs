//! Maximum-weight perfect bipartite matching via the Kuhn–Munkres (Hungarian)
//! algorithm.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Compute a maximum-weight *perfect* matching of a complete bipartite graph
/// with `n` left and `n` right vertices, using the Kuhn–Munkres algorithm.
///
/// `w` is a flat, row-major `n × n` weight matrix: `w[idx(u, n, v)]` is the
/// weight of the edge `u → v` (may be negative).  Every left vertex must be
/// matched to a distinct right vertex.
///
/// Returns `(weight, matched)` where `weight` is the maximum total weight of a
/// perfect matching and `matched` is a flat `n × n` 0/1 matrix marking the
/// matched edges.
pub fn kuhn_munkres(n: usize, w: &[i64]) -> (i64, Vec<i64>) {
    // Feasible vertex labels: `lx[u] + ly[v] ≥ w[u,v]`.
    let mut lx: Vec<i64> = vec![0i64; n];
    let mut ly: Vec<i64> = vec![0i64; n];
    let mut lx_set: Vec<bool> = vec![false; n];
    let mut matchx: Vec<usize> = vec![n; n];
    let mut matchy: Vec<usize> = vec![n; n];

    // Initial labels: `lx[u] = max_v w[u,v]`, `ly[v] = 0`.
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = 0;
        while v < n {
            let ww = w[idx(u, n, v)];
            if !lx_set[u] {
                lx[u] = ww;
                lx_set[u] = true;
            } else {
                if ww > lx[u] {
                    lx[u] = ww;
                }
            }
            v += 1;
        }
        u += 1;
    }

    // For each left vertex, find an augmenting path in the equality subgraph
    // (`lx[u] + ly[v] = w[u,v]`), adjusting labels until one exists.
    let mut u2: usize = 0;
    while u2 < n {
        while matchx[u2] == n {
            let mut visx: Vec<bool> = vec![false; n];
            let mut visy: Vec<bool> = vec![false; n];
            let mut st: Vec<usize> = vec![0usize; 2 * n];
            let mut st_len: usize = 0;
            st[0] = u2;
            st_len = 1;
            visx[u2] = true;
            let mut found: bool = false;

            // Iterative DFS over the equality subgraph.  The stack alternates
            // left/right: even entries are left vertices (raw index), odd
            // entries are right vertices (encoded as `n + v`).
            while st_len > 0 && !found {
                let u = st[st_len - 1];
                let lu = lx[u];
                let mut next_v: usize = n;
                let mut v: usize = 0;
                while v < n {
                    if !visy[v] {
                        let sum = lu + ly[v];
                        if sum == w[idx(u, n, v)] {
                            next_v = v;
                            v = n;
                        }
                    }
                    v += 1;
                }
                if next_v != n {
                    visy[next_v] = true;
                    let enc = n + next_v;
                    st[st_len] = enc;
                    st_len += 1;
                    if matchy[next_v] == n {
                        found = true;
                    } else {
                        let uu = matchy[next_v];
                        st[st_len] = uu;
                        st_len += 1;
                        visx[uu] = true;
                    }
                } else {
                    // Dead end: backtrack past `u` and the right before it.
                    st_len -= 1;
                    if st_len >= 1 {
                        st_len -= 1;
                    }
                }
            }

            if found {
                // Augment along the path: match st[2i] to st[2i+1].
                let mut i: usize = 0;
                while i + 1 < st_len {
                    let left = st[i];
                    let right = st[i + 1] - n;
                    matchx[left] = right;
                    matchy[right] = left;
                    i += 2;
                }
            } else {
                // No augmenting path in the equality subgraph: adjust labels.
                // `a = min { lx[x] + ly[y] - w[x,y] | x ∈ S, y ∉ T }`.
                let mut a: i64 = 0;
                let mut a_set: bool = false;
                let mut x: usize = 0;
                while x < n {
                    if visx[x] {
                        let mut y: usize = 0;
                        while y < n {
                            if !visy[y] {
                                let sum = lx[x] + ly[y];
                                let d = sum - w[idx(x, n, y)];
                                if !a_set {
                                    a = d;
                                    a_set = true;
                                } else {
                                    if d < a {
                                        a = d;
                                    }
                                }
                            }
                            y += 1;
                        }
                    }
                    x += 1;
                }
                // `lx[x] -= a` for x in S, `ly[y] += a` for y in T.
                let mut x2: usize = 0;
                while x2 < n {
                    if visx[x2] {
                        let new_lx = lx[x2] - a;
                        lx[x2] = new_lx;
                    }
                    x2 += 1;
                }
                let mut y2: usize = 0;
                while y2 < n {
                    if visy[y2] {
                        let new_ly = ly[y2] + a;
                        ly[y2] = new_ly;
                    }
                    y2 += 1;
                }
            }
        }
        u2 += 1;
    }

    // Build the 0/1 matching matrix and its total weight.
    let mut matched: Vec<i64> = vec![0i64; n * n];
    let mut weight: i64 = 0;
    let mut u3: usize = 0;
    while u3 < n {
        if matchx[u3] != n {
            let i = idx(u3, n, matchx[u3]);
            matched[i] = 1;
            let ww = w[i];
            weight = weight + ww;
        }
        u3 += 1;
    }
    (weight, matched)
}
