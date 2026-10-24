//! Maximum common independent set of the graphic matroid (acyclic edge sets)
//! and the partition matroid (each colour used at most `k` times).
//!
//! This is Edmonds' matroid intersection theorem made algorithmic: the
//! augmenting structure is the *exchange digraph*, and a maximum common
//! independent set is built up one element at a time by BFS along
//! augmenting paths of that digraph.
//!
//! Convention (the three ingredients must be mutually consistent):
//!
//! * the BFS *sources* are the unselected `y` with `S + y` colour-feasible
//!   in the partition matroid M2;
//! * the BFS *sinks* are the unselected `y` with `S + y` acyclic in the
//!   graphic matroid M1;
//! * the exchange arcs are `y -> x` (free -> selected) when `S - x + y` is
//!   acyclic in M1, and `x -> y` (selected -> free) when `S - x + y` is
//!   colour-feasible in M2.
//!
//! With this pairing the free -> selected arc tests the *same* matroid as
//! the sink test (M1) and the selected -> free arc tests the same matroid as
//! the source test (M2); this consistency is exactly what makes the
//! symmetric-difference flip along a shortest source-to-sink path land in
//! `I1 ∩ I2`.

/// True iff `x == 0`, as a `bool` *value* (so it can be combined with `&&`).
fn is_zero(x: usize) -> bool {
    if x == 0 { true } else { false }
}

/// True iff `a == b`, as a `bool` value.
fn same_usize(a: usize, b: usize) -> bool {
    if a == b { true } else { false }
}

/// True iff `a <= b`, as a `bool` value.
fn le(a: usize, b: usize) -> bool {
    if a <= b { true } else { false }
}

/// Number of selected elements (`sel[i] != 0`) whose colour equals `c`.
fn color_count(colors: &[usize], sel: &[usize], m: usize, c: usize) -> usize {
    let mut cnt: usize = 0;
    let mut i: usize = 0;
    while i < m {
        let s = sel[i] != 0;
        let same = same_usize(colors[i], c);
        let inc = s && same;
        if inc {
            cnt = cnt + 1;
        }
        i += 1;
    }
    cnt
}

/// Test whether the edge set `{ e : sel[e] != 0, e != skip }` together with the
/// single extra edge `extra` is acyclic (a forest) on `nv` vertices.
///
/// `skip = m` skips nothing; `extra = m` adds nothing.  Acyclicity is decided
/// with a union-find rebuilt from scratch; a self-loop or an edge whose
/// endpoints are already in the same component closes a cycle.
fn can_add_acyclic(
    nv: usize,
    m: usize,
    from: &[usize],
    to: &[usize],
    sel: &[usize],
    skip: usize,
    extra: usize,
) -> bool {
    let mut parent: Vec<usize> = vec![0usize; nv];
    let mut i: usize = 0;
    while i < nv {
        parent[i] = i;
        i += 1;
    }
    let mut e: usize = 0;
    while e < m {
        let s = sel[e] != 0;
        let notskip = e != skip;
        let do_union = s && notskip;
        if do_union {
            let u = from[e];
            let v = to[e];
            let mut ru = u;
            while parent[ru] != ru {
                ru = parent[ru];
            }
            let mut rv = v;
            while parent[rv] != rv {
                rv = parent[rv];
            }
            parent[ru] = rv;
        }
        e += 1;
    }
    let u2 = from[extra];
    let v2 = to[extra];
    let mut ok: bool = true;
    if u2 == v2 {
        // A self-loop is always a circuit.
        ok = false;
    } else {
        let mut ru = u2;
        while parent[ru] != ru {
            ru = parent[ru];
        }
        let mut rv = v2;
        while parent[rv] != rv {
            rv = parent[rv];
        }
        if ru == rv {
            ok = false;
        }
    }
    ok
}

/// Test whether `S - x + y` is colour-feasible for the partition matroid, where
/// `x` is selected and `y` is not.  Only colour `colors[y]` can overflow.
fn color_swap_ok(
    colors: &[usize],
    sel: &[usize],
    m: usize,
    x: usize,
    y: usize,
    k: usize,
) -> bool {
    let cx = colors[x];
    let cy = colors[y];
    let mut ok: bool = true;
    if cx != cy {
        let cnt = color_count(colors, sel, m, cy);
        let cnt1 = cnt + 1;
        ok = le(cnt1, k);
    }
    ok
}

/// Test whether `S + y` is colour-feasible for the partition matroid, where
/// `y` is not selected.  Only colour `colors[y]` can overflow.
fn color_add_ok(colors: &[usize], sel: &[usize], m: usize, y: usize, k: usize) -> bool {
    let cy = colors[y];
    let cnt = color_count(colors, sel, m, cy);
    let cnt1 = cnt + 1;
    le(cnt1, k)
}

/// Maximum common independent set of the graphic matroid (acyclic) and the
/// partition matroid (each colour used at most `k` times).
///
/// The ground set is the `m` directed edges of a graph on `nv` vertices;
/// `from[e]` / `to[e]` are the endpoints of edge `e` and `colors[e]` is its
/// colour.  Returns a flat 0/1 indicator of length `m` marking the chosen
/// edges.
pub fn matroid_intersection(
    nv: usize,
    m: usize,
    from: &[usize],
    to: &[usize],
    colors: &[usize],
    k: usize,
) -> Vec<usize> {
    // sel[e] = 1 iff edge e is currently in the common independent set.
    let mut sel: Vec<usize> = vec![0usize; m];

    let mut progress: bool = true;
    while progress {
        progress = false;

        // Exchange-graph BFS bookkeeping.
        let mut visited: Vec<usize> = vec![0usize; m];
        // parent[e] = predecessor of e on the BFS tree; `m` marks a source.
        let mut parent: Vec<usize> = vec![m; m];
        let mut queue: Vec<usize> = vec![0usize; m];
        let mut head: usize = 0;
        let mut tail: usize = 0;

        // Sources: unselected `y` with `S + y` colour-feasible (M2).
        let mut y: usize = 0;
        while y < m {
            let free = is_zero(sel[y]);
            if free {
                let feas = color_add_ok(colors, &sel, m, y, k);
                if feas {
                    visited[y] = 1;
                    queue[tail] = y;
                    tail += 1;
                }
            }
            y += 1;
        }

        let mut sink: usize = m;
        while head < tail {
            let u = queue[head];
            head += 1;
            let u_taken = sel[u] != 0;
            let u_free = is_zero(sel[u]);

            // A sink: unselected `u` with `S + u` acyclic (M1).
            let acy = can_add_acyclic(nv, m, from, to, &sel, m, u);
            let sink_found = u_free && acy;

            if sink_found {
                sink = u;
                head = tail; // stop the BFS
            } else {
                if u_taken {
                    // Arcs u -> v for v unselected with `S - u + v` M2-feasible.
                    let mut v: usize = 0;
                    while v < m {
                        let v_free = is_zero(sel[v]);
                        let v_vis = is_zero(visited[v]);
                        let explore = v_free && v_vis;
                        if explore {
                            let swap_ok = color_swap_ok(colors, &sel, m, u, v, k);
                            if swap_ok {
                                visited[v] = 1;
                                parent[v] = u;
                                queue[tail] = v;
                                tail += 1;
                            }
                        }
                        v += 1;
                    }
                } else {
                    // Arcs u -> v for v selected with `S - v + u` M1-acyclic.
                    let mut v: usize = 0;
                    while v < m {
                        let v_taken = sel[v] != 0;
                        let v_vis = is_zero(visited[v]);
                        let explore = v_taken && v_vis;
                        if explore {
                            let swap_ok = can_add_acyclic(nv, m, from, to, &sel, v, u);
                            if swap_ok {
                                visited[v] = 1;
                                parent[v] = u;
                                queue[tail] = v;
                                tail += 1;
                            }
                        }
                        v += 1;
                    }
                }
            }
        }

        if sink != m {
            // Flip S along the augmenting path sink -> ... -> source.
            progress = true;
            let mut cur = sink;
            while cur != m {
                let old = sel[cur];
                let nw = if old == 0 { 1usize } else { 0usize };
                sel[cur] = nw;
                cur = parent[cur];
            }
        }
    }

    sel
}
