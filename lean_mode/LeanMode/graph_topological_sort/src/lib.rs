//! Topological sort (Kahn's algorithm) on a directed graph.
//!
//! The graph is given as a CSR-style edge list `(n, from_list, to_list)`,
//! matching the shape used by `graph_dijkstra` / `graph_bellman_ford`.
//! This keeps the representation flat (no `Vec<Vec<usize>>`), which AENEAS
//! translates cleanly.

/// Compute a topological ordering of a directed graph using Kahn's algorithm
/// (repeatedly remove a zero-indegree vertex).
///
/// Returns `Some(order)` where `order` is a valid topological ordering
/// (for every edge `u → v`, `u` appears before `v` in `order`), or `None` if
/// the graph contains a cycle.
pub fn topological_sort(
    n: usize,
    from_list: &[usize],
    to_list: &[usize],
) -> Option<Vec<usize>> {
    let m = from_list.len();

    // indegree of each vertex
    let mut indeg: Vec<usize> = vec![0usize; n];
    let mut k: usize = 0;
    while k < m {
        let v = to_list[k];
        let t = indeg[v] + 1;
        indeg[v] = t;
        k += 1;
    }

    // Queue of zero-indegree vertices (fixed-size buffer + head/tail cursors).
    let mut queue: Vec<usize> = vec![0usize; n];
    let mut head: usize = 0;
    let mut tail: usize = 0;
    let mut i: usize = 0;
    while i < n {
        if indeg[i] == 0 {
            queue[tail] = i;
            tail += 1;
        }
        i += 1;
    }

    let mut order: Vec<usize> = Vec::new();

    while head < tail {
        let u = queue[head];
        head += 1;
        order.push(u);

        let mut k: usize = 0;
        while k < m {
            if from_list[k] == u {
                let v = to_list[k];
                let t = indeg[v] - 1;
                indeg[v] = t;
                if t == 0 {
                    queue[tail] = v;
                    tail += 1;
                }
            }
            k += 1;
        }
    }

    if order.len() == n {
        Some(order)
    } else {
        None
    }
}
