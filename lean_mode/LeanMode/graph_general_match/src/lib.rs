//! Maximum matching in a general (non-bipartite) graph via Edmonds' blossom
//! algorithm.

/// Flat index for a row-major `n × n` matrix.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Least common ancestor of two vertices in the alternating forest.
fn lca(n: usize, base: &[usize], mch: &[usize], p: &[usize], a0: usize, b0: usize) -> usize {
    let mut mark: Vec<bool> = vec![false; n];
    let mut a: usize = a0;
    let mut done: bool = false;
    while !done {
        a = base[a];
        mark[a] = true;
        if mch[a] == n {
            done = true;
        } else {
            let m = mch[a];
            let parent = p[m];
            a = parent;
        }
    }
    let mut b: usize = b0;
    let mut result: usize = n;
    let mut found: bool = false;
    while !found {
        b = base[b];
        if mark[b] {
            result = b;
            found = true;
        } else {
            let m = mch[b];
            let parent = p[m];
            b = parent;
        }
    }
    result
}

/// Mark the vertices on the path from `v` to base `b`, setting blossom flags
/// and rewriting parents.
fn mark_path(
    n: usize,
    base: &[usize],
    mch: &[usize],
    mut p: Vec<usize>,
    mut blossom: Vec<bool>,
    v0: usize,
    b: usize,
    children0: usize,
) -> (Vec<usize>, Vec<bool>) {
    let mut v: usize = v0;
    let mut children: usize = children0;
    while base[v] != b {
        let bv = base[v];
        blossom[bv] = true;
        let m = mch[v];
        let bm = base[m];
        blossom[bm] = true;
        p[v] = children;
        children = m;
        let parent = p[m];
        v = parent;
    }
    (p, blossom)
}

/// Find an augmenting path starting at the free vertex `root`.  Returns
/// `(endpoint, p)` where `endpoint` is the free vertex at the far end of the
/// augmenting path (`n` when none exists) and `p` is the BFS parent array.
fn find_path(
    n: usize,
    adj: &[i64],
    mch: &[usize],
    root: usize,
) -> (usize, Vec<usize>) {
    let mut used: Vec<bool> = vec![false; n];
    let mut p: Vec<usize> = vec![n; n];
    let mut base: Vec<usize> = vec![0usize; n];
    let mut i: usize = 0;
    while i < n {
        base[i] = i;
        i += 1;
    }
    let mut blossom: Vec<bool> = vec![false; n];
    let mut queue: Vec<usize> = vec![0usize; n];
    let mut head: usize = 0;
    let mut tail: usize = 0;
    queue[0] = root;
    tail = 1;
    used[root] = true;
    let mut endpoint: usize = n;

    while head < tail && endpoint == n {
        let v = queue[head];
        head += 1;
        let mut u: usize = 0;
        while u < n && endpoint == n {
            let is_edge = adj[idx(v, n, u)] == 1;
            let same_base = base[v] == base[u];
            let is_match = mch[v] == u;
            if is_edge && !same_base && !is_match {
                let mu = mch[u];
                let mut mu_parent: usize = n;
                if mu != n {
                    mu_parent = p[mu];
                }
                let is_blossom = (u == root) || (mu != n && mu_parent != n);
                if is_blossom {
                    let curbase = lca(n, &base, mch, &p, v, u);
                    let mut j: usize = 0;
                    while j < n {
                        blossom[j] = false;
                        j += 1;
                    }
                    let (p1, bl1) = mark_path(n, &base, mch, p, blossom, v, curbase, u);
                    p = p1;
                    blossom = bl1;
                    let (p2, bl2) = mark_path(n, &base, mch, p, blossom, u, curbase, v);
                    p = p2;
                    blossom = bl2;
                    let mut j2: usize = 0;
                    while j2 < n {
                        let bj = base[j2];
                        if blossom[bj] {
                            base[j2] = curbase;
                            if !used[j2] {
                                used[j2] = true;
                                queue[tail] = j2;
                                tail += 1;
                            }
                        }
                        j2 += 1;
                    }
                } else if p[u] == n {
                    p[u] = v;
                    if mu == n {
                        endpoint = u;
                    } else {
                        let u2 = mch[u];
                        used[u2] = true;
                        queue[tail] = u2;
                        tail += 1;
                    }
                }
            }
            u += 1;
        }
    }
    (endpoint, p)
}

/// Compute a maximum matching of a general undirected graph via Edmonds'
/// blossom algorithm.
///
/// `adj` is a flat, row-major, symmetric `n × n` 0/1 adjacency matrix
/// (`adj[u,v] == 1` when `u` and `v` are adjacent; no self-loops).
///
/// Returns `(size, matched)` where `size` is the matching cardinality and
/// `matched` is a flat `n × n` 0/1 matrix with `matched[u,v] == 1` on matched
/// edges.
pub fn blossom(n: usize, adj: &[i64]) -> (i64, Vec<i64>) {
    let mut mch: Vec<usize> = vec![n; n];

    let mut i: usize = 0;
    while i < n {
        if mch[i] == n {
            let (endpoint, p) = find_path(n, adj, &mch, i);
            if endpoint != n {
                let mut v: usize = endpoint;
                while v != n {
                    let pv = p[v];
                    let ppv = mch[pv];
                    mch[v] = pv;
                    mch[pv] = v;
                    v = ppv;
                }
            }
        }
        i += 1;
    }

    let mut matched: Vec<i64> = vec![0i64; n * n];
    let mut size: i64 = 0;
    let mut u: usize = 0;
    while u < n {
        if mch[u] != n {
            let v = mch[u];
            let i2 = idx(u, n, v);
            matched[i2] = 1;
            if u < v {
                size = size + 1;
            }
        }
        u += 1;
    }
    (size, matched)
}
