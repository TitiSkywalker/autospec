//! Brooks' theorem (1941), constructively.
//!
//! Every connected graph that is neither complete nor an odd cycle can be
//! properly coloured with `Δ` colours, where `Δ` is its maximum degree; a
//! complete graph needs `Δ + 1 = n` colours and an odd cycle needs `3 = Δ + 1`.
//!
//! `brooks_color` returns such a colouring `col` (length `n`, `col[v] ∈
//! 0..used`) for a graph given as a flat row-major `n × n` symmetric 0/1
//! adjacency matrix:
//!
//! 1. complete graph → colour `v` with `v` (`n = Δ + 1` colours),
//! 2. `Δ ≤ 2` → every component is a path or a cycle: BFS 2-colouring; if a
//!    component is an odd cycle, fall back to the `Δ + 1 = 3` colour scheme,
//! 3. `Δ ≥ 3` and not complete → find a vertex `v` with two non-adjacent
//!    neighbours `u, w` such that `G − {u, w}` is connected, and use the
//!    Brooks ordering: colour `u` and `w` alike, then the vertices of
//!    `G − {u, w}` in reverse BFS order from `v`, with `v` last.  Every
//!    vertex but `v` has a later neighbour (its BFS parent) and `v` sees at
//!    most `Δ − 1` colours (because `u, w` share one), so `Δ` colours suffice,
//! 4. otherwise → exhaustive (iterative, stack-free) backtracking search for a
//!    proper `Δ`-colouring, falling back to first-fit greedy with `Δ + 1`
//!    colours (which always succeeds).

/// Flat row-major index of entry `(row, col)` in an `n × n` matrix.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Maximum degree `Δ` of the graph.
fn max_degree(n: usize, adj: &[usize]) -> usize {
    let mut d: usize = 0;
    let mut v: usize = 0;
    while v < n {
        let mut deg: usize = 0;
        let mut u: usize = 0;
        while u < n {
            let a = adj[idx(u, n, v)];
            if a == 1 {
                deg += 1;
            }
            u += 1;
        }
        if deg > d {
            d = deg;
        }
        v += 1;
    }
    d
}

/// Is the graph complete, i.e. is every pair of distinct vertices adjacent?
fn is_complete_graph(n: usize, adj: &[usize]) -> bool {
    let mut complete: bool = true;
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = 0;
        while v < n {
            if u != v {
                let a = adj[idx(u, n, v)];
                if a == 0 {
                    complete = false;
                }
            }
            v += 1;
        }
        u += 1;
    }
    complete
}

/// 2-colour every component by BFS.  Returns `(conflict, color)`: `conflict`
/// is `true` iff some component contains an odd cycle.  When `conflict` is
/// `false` every entry of `color` is `0` or `1` and the colouring is proper.
fn two_color(n: usize, adj: &[usize]) -> (bool, Vec<usize>) {
    let mut color: Vec<usize> = vec![2usize; n];
    let mut queue: Vec<usize> = vec![0usize; n];
    let mut conflict: bool = false;

    let mut s: usize = 0;
    while s < n {
        if color[s] == 2 {
            color[s] = 0;
            let mut head: usize = 0;
            let mut tail: usize = 0;
            queue[tail] = s;
            tail += 1;

            while head < tail {
                let u = queue[head];
                head += 1;
                let cu = color[u];

                let mut v: usize = 0;
                while v < n {
                    let a = adj[idx(u, n, v)];
                    if a == 1 {
                        let cv = color[v];
                        if cv == 2 {
                            let opposite = 1 - cu;
                            color[v] = opposite;
                            queue[tail] = v;
                            tail += 1;
                        } else {
                            let cv2 = color[v];
                            if cv2 == cu {
                                conflict = true;
                            }
                        }
                    }
                    v += 1;
                }
            }
        }
        s += 1;
    }

    (conflict, color)
}

/// First-fit greedy colouring with palette `0..k`.  Vertices are coloured in
/// index order, so only the already-coloured `u < v` are inspected.  Whenever
/// `k > Δ` a free colour always exists for the current vertex.
fn greedy_coloring(n: usize, adj: &[usize], k: usize) -> Vec<usize> {
    let mut col: Vec<usize> = vec![0usize; n];
    // one slot more than the largest possible degree, so that `mark[c]` is in
    // range even if the matrix carries self-loops (`Δ` can then be `n`)
    let mut mark: Vec<usize> = vec![0usize; n + 1];

    let mut v: usize = 0;
    while v < n {
        let mut t: usize = 0;
        while t < k {
            mark[t] = 0;
            t += 1;
        }

        let mut u: usize = 0;
        while u < v {
            let a = adj[idx(u, n, v)];
            if a == 1 {
                let cu = col[u];
                if cu < k {
                    mark[cu] = 1;
                }
            }
            u += 1;
        }

        let mut c: usize = k;
        let mut t2: usize = 0;
        while t2 < k {
            let m = mark[t2];
            if m == 0 {
                if t2 < c {
                    c = t2;
                }
            }
            t2 += 1;
        }

        col[v] = c;
        v += 1;
    }

    col
}

/// Is `G − {u, w}` connected, explored by BFS from the remaining vertex `v`?
/// `u` and `w` are treated as removed; the test is whether all `n − 2`
/// remaining vertices are reached.
fn minus_pair_connected(n: usize, adj: &[usize], v: usize, u: usize, w: usize) -> bool {
    let mut seen: Vec<usize> = vec![0usize; n];
    let mut queue: Vec<usize> = vec![0usize; n];
    seen[v] = 1;
    queue[0] = v;
    let mut head: usize = 0;
    let mut tail: usize = 1;
    let mut count: usize = 1;

    while head < tail {
        let x = queue[head];
        head += 1;
        let mut y: usize = 0;
        while y < n {
            let a = adj[idx(x, n, y)];
            if a == 1 {
                let s = seen[y];
                if s == 0 {
                    if y != u {
                        if y != w {
                            seen[y] = 1;
                            queue[tail] = y;
                            tail += 1;
                            count += 1;
                        }
                    }
                }
            }
            y += 1;
        }
    }

    let total = count + 2;
    if total == n { true } else { false }
}

/// Does `v` have two non-adjacent neighbours `u < w` such that `G − {u, w}`
/// stays connected?  Such a triple is exactly what the Brooks ordering needs.
fn brooks_pair_valid(n: usize, adj: &[usize], v: usize, u: usize, w: usize) -> bool {
    let mut valid: bool = false;
    if u < w {
        let a1 = adj[idx(v, n, u)];
        let a2 = adj[idx(v, n, w)];
        if a1 == 1 {
            if a2 == 1 {
                let a3 = adj[idx(u, n, w)];
                if a3 == 0 {
                    let conn = minus_pair_connected(n, adj, v, u, w);
                    if conn {
                        valid = true;
                    }
                }
            }
        }
    }
    valid
}

/// The first vertex `v` with witnesses `u < w` of the Brooks ordering
/// (`brooks_pair_valid`); a return value of `n` for `v` means "none found".
fn find_brooks_pair(n: usize, adj: &[usize]) -> (usize, usize, usize) {
    // The triple is accumulated in an array: the three witnesses must be
    // updated together, and Aeneas' loop analysis dislikes several mutually
    // assigned scalars (it abandons the whole body).
    let mut best: Vec<usize> = vec![0usize; 3];
    best[0] = n;

    let mut v: usize = 0;
    while v < n {
        let mut u: usize = 0;
        while u < n {
            let mut w: usize = u + 1;
            while w < n {
                let valid = brooks_pair_valid(n, adj, v, u, w);
                if valid {
                    let fv = best[0];
                    if v < fv {
                        best[0] = v;
                        best[1] = u;
                        best[2] = w;
                    }
                }
                w += 1;
            }
            u += 1;
        }
        v += 1;
    }

    let fv = best[0];
    let fu = best[1];
    let fw = best[2];
    (fv, fu, fw)
}

/// Brooks ordering with respect to the witnesses `(fv, fu, fw)`: colour `fu`
/// and `fw` alike, then all vertices of `G − {fu, fw}` in reverse BFS order
/// from `fv`, and `fv` last.  Uses only colours `0..delta`.
fn brooks_ordered_coloring(
    n: usize,
    adj: &[usize],
    delta: usize,
    fv: usize,
    fu: usize,
    fw: usize,
) -> Vec<usize> {
    // BFS order of `G − {fu, fw}` from `fv`; a FIFO queue lists vertices in
    // BFS order, so the queue itself is the order.
    let m2 = n - 2;
    let mut queue: Vec<usize> = vec![0usize; m2];
    let mut seen: Vec<usize> = vec![0usize; n];
    queue[0] = fv;
    seen[fv] = 1;
    let mut head: usize = 0;
    let mut tail: usize = 1;

    while head < tail {
        let x = queue[head];
        head += 1;
        let mut y: usize = 0;
        while y < n {
            let a = adj[idx(x, n, y)];
            if a == 1 {
                let s = seen[y];
                if s == 0 {
                    if y != fu {
                        if y != fw {
                            seen[y] = 1;
                            queue[tail] = y;
                            tail += 1;
                        }
                    }
                }
            }
            y += 1;
        }
    }

    let mut col: Vec<usize> = vec![0usize; n];
    let mut placed: Vec<usize> = vec![0usize; n];
    col[fu] = 0;
    placed[fu] = 1;
    col[fw] = 0;
    placed[fw] = 1;
    let mut mark: Vec<usize> = vec![0usize; delta];

    // Reverse BFS order, the last vertex to be coloured being `queue[0] = fv`.
    let mut i: usize = m2;
    while i > 0 {
        i -= 1;
        let x = queue[i];

        let mut t: usize = 0;
        while t < delta {
            mark[t] = 0;
            t += 1;
        }

        let mut y: usize = 0;
        while y < n {
            let a = adj[idx(x, n, y)];
            if a == 1 {
                let p = placed[y];
                if p == 1 {
                    let cy = col[y];
                    if cy < delta {
                        mark[cy] = 1;
                    }
                }
            }
            y += 1;
        }

        let mut c: usize = delta;
        let mut t2: usize = 0;
        while t2 < delta {
            let m = mark[t2];
            if m == 0 {
                if t2 < c {
                    c = t2;
                }
            }
            t2 += 1;
        }

        col[x] = c;
        placed[x] = 1;
    }

    col
}

/// Exhaustive backtracking search for a proper colouring with `k` colours.
/// Iterative and stack-free: `nxt[v]` is the next candidate colour to try at
/// depth `v`, `val[v]` the currently assigned one.  Returns `(found, color)`.
fn backtrack_coloring(n: usize, adj: &[usize], k: usize) -> (bool, Vec<usize>) {
    let mut val: Vec<usize> = vec![0usize; n];
    let mut nxt: Vec<usize> = vec![0usize; n];
    let mut found: bool = false;
    let mut dead: bool = false;
    let mut i: usize = 0;

    while i < n {
        if dead {
            i = n;
        } else {
            // first colour in `nxt[i]..k` that is free at vertex `i`
            let mut chosen: usize = k;
            let mut c: usize = nxt[i];
            while c < k {
                let mut okc: bool = true;
                let mut j: usize = 0;
                while j < i {
                    let a = adj[idx(j, n, i)];
                    if a == 1 {
                        let cj = val[j];
                        if cj == c {
                            okc = false;
                        }
                    }
                    j += 1;
                }
                if okc {
                    if c < chosen {
                        chosen = c;
                    }
                }
                c += 1;
            }

            if chosen < k {
                val[i] = chosen;
                let nc = chosen + 1;
                nxt[i] = nc;
                let ni = i + 1;
                if ni == n {
                    found = true;
                }
                i = ni;
            } else {
                // vertex `i` has no admissible colour left: step back
                nxt[i] = 0;
                if i == 0 {
                    dead = true;
                } else {
                    i -= 1;
                }
            }
        }
    }

    (found, val)
}

/// A proper colouring of the graph `adj` (flat row-major `n × n` 0/1 symmetric
/// adjacency matrix) with at most `Δ + 1` colours, and at most `Δ` colours
/// whenever the graph is connected, not complete and not an odd cycle.
pub fn brooks_color(n: usize, adj: &[usize]) -> Vec<usize> {
    let mut col: Vec<usize> = vec![0usize; n];

    if n > 1 {
        let complete = is_complete_graph(n, adj);
        if complete {
            // the extremal case: K_n needs n = Δ + 1 colours
            let mut i: usize = 0;
            while i < n {
                col[i] = i;
                i += 1;
            }
        } else {
            let delta = max_degree(n, adj);
            if delta <= 2 {
                let (conflict, c2) = two_color(n, adj);
                if conflict {
                    // an odd cycle needs a third colour
                    col = greedy_coloring(n, adj, delta + 1);
                } else {
                    col = c2;
                }
            } else {
                let (fv, fu, fw) = find_brooks_pair(n, adj);
                if fv < n {
                    col = brooks_ordered_coloring(n, adj, delta, fv, fu, fw);
                } else {
                    let (found, val) = backtrack_coloring(n, adj, delta);
                    if found {
                        col = val;
                    } else {
                        col = greedy_coloring(n, adj, delta + 1);
                    }
                }
            }
        }
    }

    col
}
