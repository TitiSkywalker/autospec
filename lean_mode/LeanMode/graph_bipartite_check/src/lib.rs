//! Bipartite (2-coloring) check via BFS on an undirected graph.
//!
//! The graph is given as a CSR-style edge list `(n, from_list, to_list)`,
//! where each *undirected* edge `{u, v}` appears exactly once (matching the
//! convention of `graph_kruskal` / `graph_prim`).

/// Decide whether an undirected graph is bipartite by BFS 2-coloring.
///
/// Returns `Some(colors)` where `colors[v] ∈ {0, 1}` and every edge connects
/// vertices of different colors, or `None` if no such 2-coloring exists
/// (i.e. the graph contains an odd cycle).
///
/// Internally the color vector uses `2` as the "uncolored" sentinel; the
/// returned vector only contains `0` / `1`.
pub fn is_bipartite(
    n: usize,
    from_list: &[usize],
    to_list: &[usize],
) -> Option<Vec<usize>> {
    let m = from_list.len();

    // color: 0 / 1 = the two sides, 2 = uncolored
    let mut color: Vec<usize> = vec![2usize; n];
    let mut queue: Vec<usize> = vec![0usize; n];
    let mut conflict: bool = false;

    let mut s: usize = 0;
    while s < n {
        if color[s] == 2 {
            // Start a new BFS component, colored 0.
            color[s] = 0;
            let mut head: usize = 0;
            let mut tail: usize = 0;
            queue[tail] = s;
            tail += 1;

            while head < tail {
                let u = queue[head];
                head += 1;
                let cu = color[u];

                let mut k: usize = 0;
                while k < m {
                    let a = from_list[k];
                    let b = to_list[k];

                    if a == u {
                        let cb = color[b];
                        if cb == 2 {
                            let opposite = 1 - cu;
                            color[b] = opposite;
                            queue[tail] = b;
                            tail += 1;
                        } else {
                            let cb2 = color[b];
                            if cb2 == cu {
                                conflict = true;
                            }
                        }
                    }

                    if b == u {
                        let ca = color[a];
                        if ca == 2 {
                            let opposite = 1 - cu;
                            color[a] = opposite;
                            queue[tail] = a;
                            tail += 1;
                        } else {
                            let ca2 = color[a];
                            if ca2 == cu {
                                conflict = true;
                            }
                        }
                    }

                    k += 1;
                }
            }
        }
        s += 1;
    }

    if conflict {
        None
    } else {
        Some(color)
    }
}
