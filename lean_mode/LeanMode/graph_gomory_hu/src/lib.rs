//! Gomory–Hu cut tree (Gusfield's algorithm) of an undirected graph given as a
//! flat, symmetric capacity matrix.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Edmonds–Karp maximum flow from `s` to `t` (Ford–Fulkerson with BFS shortest
/// augmenting paths) together with the *source side* of a minimum `s`–`t` cut.
///
/// The graph is the flat, row-major `n × n` capacity matrix `cap` (entry
/// `idx(u, n, v)` is the capacity of the edge `u — v`); the flow is computed on
/// an explicit residual capacity matrix, so every residual value is
/// non-negative and an undirected edge contributes capacity in both
/// directions.
///
/// Returns `(value, side)` where `value` is the maximum flow value, which by
/// the max-flow/min-cut theorem equals the minimum `s`–`t` cut value, and
/// `side[v] == 1` iff `v` is reachable from `s` in the final residual graph —
/// i.e. `side` is the indicator of the source side of a minimum cut.
fn maxflow_mincut(n: usize, cap: &[usize], s: usize, t: usize) -> (usize, Vec<usize>) {
    let nn = n * n;
    // Residual capacities, initialised to the capacities themselves.
    let mut res: Vec<usize> = vec![0usize; nn];
    let mut i: usize = 0;
    while i < nn {
        res[i] = cap[i];
        i += 1;
    }

    let mut value: usize = 0;

    let mut progress: bool = s != t;
    while progress {
        // BFS from `s` along residual edges (`residual > 0`); `parent[v] == n`
        // means "not discovered yet".
        let mut parent: Vec<usize> = vec![n; n];
        let mut queue: Vec<usize> = vec![0usize; n];
        parent[s] = s;
        queue[0] = s;
        let mut head: usize = 0;
        let mut tail: usize = 1;

        while head < tail {
            let u = queue[head];
            head += 1;
            let mut v: usize = 0;
            while v < n {
                if parent[v] == n {
                    let r = res[idx(u, n, v)];
                    if r > 0 {
                        parent[v] = u;
                        queue[tail] = v;
                        tail += 1;
                    }
                }
                v += 1;
            }
        }

        if parent[t] == n {
            // No augmenting path left: the flow is maximum.
            progress = false;
        } else {
            // Bottleneck = minimum residual capacity along the path `t → s`.
            let mut b: usize = 0;
            let mut b_set: bool = false;
            let mut v: usize = t;
            while v != s {
                let u = parent[v];
                let r = res[idx(u, n, v)];
                if !b_set {
                    b = r;
                    b_set = true;
                } else {
                    if r < b {
                        b = r;
                    }
                }
                v = u;
            }

            // Augment `b` units along the path (forward and reverse).
            let mut v2: usize = t;
            while v2 != s {
                let u = parent[v2];
                let ifwd = idx(u, n, v2);
                let irev = idx(v2, n, u);
                let rf = res[ifwd];
                let rr = res[irev];
                let nf = rf - b;
                let nr = rr + b;
                res[ifwd] = nf;
                res[irev] = nr;
                v2 = u;
            }

            let nvalue = value + b;
            value = nvalue;
        }
    }

    // Source side of a minimum cut: every vertex reachable from `s` in the
    // final residual graph.
    let mut side: Vec<usize> = vec![0usize; n];
    let mut queue2: Vec<usize> = vec![0usize; n];
    side[s] = 1;
    queue2[0] = s;
    let mut head2: usize = 0;
    let mut tail2: usize = 1;
    while head2 < tail2 {
        let u = queue2[head2];
        head2 += 1;
        let mut v: usize = 0;
        while v < n {
            if side[v] == 0 {
                let r = res[idx(u, n, v)];
                if r > 0 {
                    side[v] = 1;
                    queue2[tail2] = v;
                    tail2 += 1;
                }
            }
            v += 1;
        }
    }

    (value, side)
}

/// Build a Gomory–Hu cut tree of the undirected graph whose flat, row-major
/// `n × n` capacity matrix is `cap` (`cap[u, v] = cap[v, u]`, `0` = no edge),
/// using Gusfield's algorithm: `n - 1` maximum-flow computations, each on the
/// original graph.
///
/// Returns `(parent, weight)`, the tree rooted at vertex `0`: `parent[0] = 0`
/// and for `i ≠ 0` the tree edge `(i, parent[i])` has capacity `weight[i]`.
/// The tree satisfies: for all `u ≠ v`, the minimum `u`–`v` cut value in `cap`
/// equals the minimum `weight` on the tree path from `u` to `v`.
pub fn gomory_hu(n: usize, cap: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let mut parent: Vec<usize> = vec![0usize; n];
    let mut weight: Vec<usize> = vec![0usize; n];

    let mut s: usize = 1;
    while s < n {
        let t = parent[s];
        let (f, side) = maxflow_mincut(n, cap, s, t);

        // Every vertex still attached to `t` that lies on `s`'s side of the
        // cut is re-attached to `s`.
        let mut i: usize = s + 1;
        while i < n {
            if parent[i] == t {
                if side[i] == 1 {
                    parent[i] = s;
                }
            }
            i += 1;
        }

        if side[t] == 1 {
            // `t` lies on `s`'s side of the cut: `s` takes `t`'s place.
            let pt = parent[t];
            let wt = weight[t];
            parent[s] = pt;
            weight[s] = wt;
            parent[t] = s;
            weight[t] = f;
        } else {
            weight[s] = f;
        }

        s += 1;
    }

    (parent, weight)
}
