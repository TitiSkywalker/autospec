//! Dijkstra's shortest path algorithm (O(V²) with adjacency edge list).
//! Uses integer weights to avoid Aeneas f64 comparison limitations.

/// Compute shortest distances from `start` to all vertices.
///
/// Returns `dist[v]` = shortest distance from start to v (as integer).
/// Unreachable vertices have `has_dist[v] = false` (callers must track separately).
pub fn dijkstra(
    n: usize,
    from_list: &[usize],
    to_list: &[usize],
    weight_list: &[i64],
    start: usize,
) -> Vec<i64> {
    let mut dist: Vec<i64> = Vec::new();
    let mut has_dist: Vec<bool> = Vec::new();
    let mut visited: Vec<bool> = Vec::new();

    let mut i: usize = 0;
    while i < n {
        dist.push(0);
        has_dist.push(false);
        visited.push(false);
        i += 1;
    }
    dist[start] = 0;
    has_dist[start] = true;

    let edges_len = from_list.len();

    let mut iter: usize = 0;
    while iter < n {
        let mut u: usize = n;
        let mut min_dist: i64 = 0;
        let mut found: bool = false;

        let mut i: usize = 0;
        while i < n {
            if !visited[i] && has_dist[i] {
                if !found {
                    min_dist = dist[i];
                    u = i;
                    found = true;
                } else {
                    let di = dist[i];
                    if di < min_dist {
                        min_dist = di;
                        u = i;
                    }
                }
            }
            i += 1;
        }

        if u == n {
            break;
        }

        visited[u] = true;

        let mut k: usize = 0;
        while k < edges_len {
            if from_list[k] == u {
                let v = to_list[k];
                let w = weight_list[k];
                let new_dist = dist[u] + w;
                if !has_dist[v] {
                    dist[v] = new_dist;
                    has_dist[v] = true;
                } else if new_dist < dist[v] {
                    dist[v] = new_dist;
                }
            }
            k += 1;
        }

        iter += 1;
    }

    dist
}
