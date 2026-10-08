//! Breadth-first search (BFS) shortest path on an unweighted directed graph.
//!
//! The graph is given as a CSR-style edge list `(n, from_list, to_list)`,
//! matching the shape used by `graph_dijkstra` / `graph_topological_sort`.
//! This keeps the representation flat (no `Vec<Vec<usize>>`), which AENEAS
//! translates cleanly.

/// Compute the shortest distance (number of edges) from `start` to `target`.
///
/// Returns `Some(d)` where `d` is the minimum number of edges on a directed
/// path from `start` to `target`, or `None` if `target` is unreachable from
/// `start`.  `start == target` yields `Some(0)`.
pub fn bfs(
    n: usize,
    from_list: &[usize],
    to_list: &[usize],
    start: usize,
    target: usize,
) -> Option<usize> {
    let m = from_list.len();

    // `dist[v] == n` means "not yet discovered".
    let mut dist: Vec<usize> = vec![n; n];
    let mut queue: Vec<usize> = vec![0usize; n];

    dist[start] = 0;
    queue[0] = start;
    let mut head: usize = 0;
    let mut tail: usize = 1;

    while head < tail {
        let u = queue[head];
        head += 1;

        let mut k: usize = 0;
        while k < m {
            if from_list[k] == u {
                let v = to_list[k];
                if dist[v] == n {
                    let d = dist[u] + 1;
                    dist[v] = d;
                    queue[tail] = v;
                    tail += 1;
                }
            }
            k += 1;
        }
    }

    if dist[target] == n {
        None
    } else {
        Some(dist[target])
    }
}
