//! Bellman-Ford shortest path algorithm (O(VE) with edge list).
//! Supports negative edge weights and detects negative cycles.
//! Uses integer weights to avoid Aeneas f64 comparison limitations.

/// Compute shortest distances from `start` to all vertices using Bellman-Ford.
///
/// Returns `Ok((dist, reachable))` where:
/// - `dist[v]` = shortest distance from start to v (0 for unreachable vertices).
/// - `reachable[v]` = true iff v is reachable from start.
///
/// Returns `Err(())` if a negative cycle is detected.
pub fn bellman_ford(
    n: usize,
    from_list: &[usize],
    to_list: &[usize],
    weight_list: &[i64],
    start: usize,
) -> Result<(Vec<i64>, Vec<bool>), ()> {
    let mut dist: Vec<i64> = Vec::new();
    let mut reachable: Vec<bool> = Vec::new();

    let mut i: usize = 0;
    while i < n {
        dist.push(0);
        reachable.push(false);
        i += 1;
    }
    dist[start] = 0;
    reachable[start] = true;

    let edges_len = from_list.len();

    // Relax edges n-1 times
    let mut iter: usize = 1;
    while iter < n {
        let mut did_update: bool = false;

        let mut k: usize = 0;
        while k < edges_len {
            let u = from_list[k];
            let v = to_list[k];
            let w = weight_list[k];

            if reachable[u] {
                let new_dist = dist[u] + w;
                if !reachable[v] {
                    dist[v] = new_dist;
                    reachable[v] = true;
                    did_update = true;
                } else {
                    let dv = dist[v];
                    if new_dist < dv {
                        dist[v] = new_dist;
                        did_update = true;
                    }
                }
            }

            k += 1;
        }

        if !did_update {
            break;
        }

        iter += 1;
    }

    // Check for negative weight cycles
    let mut k: usize = 0;
    while k < edges_len {
        let u = from_list[k];
        let v = to_list[k];
        let w = weight_list[k];

        if reachable[u] {
            let dv = dist[v];
            let new_dist = dist[u] + w;
            if reachable[v] {
                if new_dist < dv {
                    return Err(());
                }
            } else {
                // reachable[u] && !reachable[v] means a path still being discovered
                // after n-1 iterations → negative cycle
                return Err(());
            }
        }

        k += 1;
    }

    Ok((dist, reachable))
}
