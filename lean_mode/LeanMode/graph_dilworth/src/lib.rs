//! Minimum vertex-disjoint path cover of a DAG — via the split bipartite
//! graph, König's theorem and Dilworth's theorem.
//!
//! A *path cover* of a DAG is a set of vertex-disjoint directed paths whose
//! vertex sets partition `0..n-1`.  The minimum number of paths in a path
//! cover equals
//!
//!   `n - (maximum matching of the split bipartite graph)`,
//!
//! where the split graph has a left copy `u_out` and a right copy `v_in` of
//! every vertex and an edge `u_out -- v_in` for every DAG edge `u -> v`.
//! (König / Dilworth: the minimum chain cover of the reachability poset is
//! `n - max matching`.)

/// Flat index for a row-major `rows × cols` matrix: element `(row, col)`
/// lives at `row * cols + col`.
fn idx(row: usize, cols: usize, col: usize) -> usize {
    let t = row * cols;
    let r = t + col;
    r
}

/// `sub_from - sub_to` on two `usize` *parameters* (the path-cover count
/// `n - matching`), isolated so no `usize` arithmetic appears inline.
fn cover_size(sub_from: usize, sub_to: usize) -> usize {
    let r = sub_from - sub_to;
    r
}

/// Size of a minimum vertex-disjoint path cover of the DAG on `0..n-1`.
///
/// The DAG is given by parallel edge lists `from` / `to`: for every index `e`,
/// `from[e] -> to[e]` is a directed edge.  Each directed edge appears once;
/// the lists must be left empty when `n == 0`.
pub fn min_path_cover(n: usize, from: &[usize], to: &[usize]) -> usize {
    let m = from.len();

    // Split-graph adjacency: row `u` is the out-copy of `u`, column `v` is the
    // in-copy of `v`.
    let mut adj: Vec<usize> = vec![0usize; n * n];
    let mut e: usize = 0;
    while e < m {
        let u = from[e];
        let v = to[e];
        let i = idx(u, n, v);
        adj[i] = 1;
        e += 1;
    }

    // match_l[u] = right copy matched to `u` (`n` = unmatched)
    // match_r[v] = left copy matched to `v` (`n` = unmatched)
    let mut match_l: Vec<usize> = vec![n; n];
    let mut match_r: Vec<usize> = vec![n; n];

    // Kuhn's algorithm, iterative: process every left copy once.
    let mut src: usize = 0;
    while src < n {
        // A left copy is matched only by augmenting paths that *start* at it,
        // so if `src` is still free an augmenting path may start here.
        if match_l[src] == n {
            let mut seen: Vec<usize> = vec![0usize; n];
            let mut reach: Vec<usize> = vec![0usize; n];
            let mut from_left: Vec<usize> = vec![0usize; n];
            let mut via_right: Vec<usize> = vec![0usize; n];
            let mut queue: Vec<usize> = vec![0usize; n];

            // `seen[v]` = right copy `v` was examined by this BFS; it is reset
            // at the start of every attempt, so nothing is pre-marked.  In
            // particular the right copy of `src` starts *unseen*: it is a
            // legitimate endpoint of an augmenting path whenever `src` has an
            // incoming edge reachable along an alternating path, and
            // pre-marking it would make the matching sub-maximal.
            queue[0] = src;
            let mut head: usize = 0;
            let mut tail: usize = 1;
            let mut free_v: usize = n;

            // BFS over the alternating graph for an augmenting path starting
            // at `src` and ending at a free right copy.
            while head < tail && free_v == n {
                let u = queue[head];
                head += 1;
                let mut v: usize = 0;
                while v < n && free_v == n {
                    let i = idx(u, n, v);
                    if adj[i] != 0 {
                        if seen[v] == 0 {
                            seen[v] = 1;
                            from_left[v] = u;
                            let w = match_r[v];
                            if w == n {
                                free_v = v;
                            } else {
                                if reach[w] == 0 {
                                    reach[w] = 1;
                                    via_right[w] = v;
                                    queue[tail] = w;
                                    tail += 1;
                                }
                            }
                        }
                    }
                    v += 1;
                }
            }

            if free_v != n {
                // Flip the matching along the augmenting path
                //      src - free_v ...
                // walking back through `via_right` until we reach `src`.
                let mut v = free_v;
                let mut done: bool = false;
                while !done {
                    let u = from_left[v];
                    match_l[u] = v;
                    match_r[v] = u;
                    if u == src {
                        done = true;
                    } else {
                        v = via_right[u];
                    }
                }
            }
        }
        src += 1;
    }

    // Cardinality of the matching we built.
    let mut size: usize = 0;
    let mut u: usize = 0;
    while u < n {
        if match_l[u] != n {
            size += 1;
        }
        u += 1;
    }
    cover_size(n, size)
}
