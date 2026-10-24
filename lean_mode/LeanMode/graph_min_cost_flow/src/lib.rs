//! Minimum-cost maximum flow via successive shortest augmenting paths with
//! potentials (Johnson's technique).

/// "Infinity" sentinel for distances.  Large enough to be unattainable by any
/// real path, small enough that `h[v] + dist` cannot overflow `i64` when the
/// weights are kept modest (as in the test suite).
const INF: i64 = 1_000_000_000_000_000;

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Dijkstra over the residual graph using *reduced costs*.
///
/// Returns `(dist_red, parent, parent_fwd)` where:
/// - `dist_red[v]` is the shortest reduced-cost distance from `source`
///   (`INF` means unreachable),
/// - `parent[v]` is the predecessor of `v` on a shortest path
///   (`parent[source] = source`),
/// - `parent_fwd[v]` records whether the residual edge `parent[v] → v` is a
///   forward edge (`true`, send more flow) or a reverse edge (`false`, cancel
///   flow on the opposite direction).
fn dijkstra(
    n: usize,
    cap: &[i64],
    cost: &[i64],
    flow: &[i64],
    h: &[i64],
    source: usize,
) -> (Vec<i64>, Vec<usize>, Vec<bool>) {
    let mut dist_red: Vec<i64> = vec![INF; n];
    let mut parent: Vec<usize> = vec![n; n];
    let mut parent_fwd: Vec<bool> = vec![false; n];
    let mut visited: Vec<bool> = vec![false; n];

    dist_red[source] = 0;
    parent[source] = source;

    let mut iter: usize = 0;
    while iter < n {
        // Pick the unvisited vertex with the smallest finite distance.
        let mut u: usize = n;
        let mut best: i64 = 0;
        let mut found: bool = false;
        let mut i: usize = 0;
        while i < n {
            if !visited[i] {
                let di = dist_red[i];
                if di < INF {
                    if !found {
                        best = di;
                        u = i;
                        found = true;
                    } else {
                        if di < best {
                            best = di;
                            u = i;
                        }
                    }
                }
            }
            i += 1;
        }
        if u == n {
            break;
        }
        visited[u] = true;

        // Relax every neighbor `v` reachable through the residual graph.
        let mut v: usize = 0;
        while v < n {
            // Forward residual edge u → v: capacity cap[u,v] - flow[u,v] > 0.
            let cap_uv = cap[idx(u, n, v)];
            let flow_uv = flow[idx(u, n, v)];
            let rcap_f = cap_uv - flow_uv;
            if rcap_f > 0 {
                let rc_f = cost[idx(u, n, v)] + h[u] - h[v];
                let nd = dist_red[u] + rc_f;
                if nd < dist_red[v] {
                    dist_red[v] = nd;
                    parent[v] = u;
                    parent_fwd[v] = true;
                }
            }
            // Reverse residual edge u → v: cancels flow on v → u (flow[v,u] > 0).
            let flow_vu = flow[idx(v, n, u)];
            if flow_vu > 0 {
                let rc_r = 0 - cost[idx(v, n, u)] + h[u] - h[v];
                let nd = dist_red[u] + rc_r;
                if nd < dist_red[v] {
                    dist_red[v] = nd;
                    parent[v] = u;
                    parent_fwd[v] = false;
                }
            }
            v += 1;
        }

        iter += 1;
    }

    (dist_red, parent, parent_fwd)
}

/// Compute a minimum-cost maximum `source`–`sink` flow via successive shortest
/// augmenting paths with potentials.
///
/// The graph is given by two flat `n × n` row-major matrices: `cap` (directed
/// edge capacities, `0` = no edge) and `cost` (per-unit cost of each edge,
/// which may be negative).  There must be no negative-cost cycle, otherwise the
/// minimum-cost flow is unbounded and the function returns `Err(())`.
///
/// Returns `Ok((value, total_cost, flow))` where `flow` is a non-negative flat
/// `n × n` matrix (`flow[u,v]` is the flow sent on `u → v`), `value` is the net
/// outflow from `source` (the maximum possible), and `total_cost` is
/// `Σ flow[u,v] * cost[u,v]`.
pub fn min_cost_flow(
    n: usize,
    cap: &[i64],
    cost: &[i64],
    source: usize,
    sink: usize,
) -> Result<(i64, i64, Vec<i64>), ()> {
    let mut flow: Vec<i64> = vec![0i64; n * n];

    // ── 1. Initial potentials via Bellman–Ford from a virtual super-source
    //       (all h[v] start at 0) so every reduced cost is non-negative, even
    //       for edges unreachable from `source`.
    let mut h: Vec<i64> = vec![0i64; n];
    let mut pass: usize = 0;
    let mut converged: bool = false;
    while pass < n && !converged {
        let mut changed: bool = false;
        let mut u: usize = 0;
        while u < n {
            let mut v: usize = 0;
            while v < n {
                let c = cap[idx(u, n, v)];
                if c > 0 {
                    let new_h = h[u] + cost[idx(u, n, v)];
                    if new_h < h[v] {
                        h[v] = new_h;
                        changed = true;
                    }
                }
                v += 1;
            }
            u += 1;
        }
        if !changed {
            converged = true;
        }
        pass += 1;
    }
    // Negative-cycle check: a further relaxation means a negative cycle.
    let mut has_neg_cycle: bool = false;
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = 0;
        while v < n {
            let c = cap[idx(u, n, v)];
            if c > 0 {
                let new_h = h[u] + cost[idx(u, n, v)];
                if new_h < h[v] {
                    has_neg_cycle = true;
                }
            }
            v += 1;
        }
        u += 1;
    }
    if has_neg_cycle {
        return Err(());
    }

    // ── 2. Successive shortest augmenting paths.
    let mut total_flow: i64 = 0;
    let mut total_cost: i64 = 0;
    let mut progress: bool = source != sink;
    while progress {
        let (dist_red, parent, parent_fwd) = dijkstra(n, cap, cost, &flow, &h, source);

        if dist_red[sink] >= INF {
            progress = false;
        } else {
            // Cost of the chosen path in *original* costs (before the potential
            // update, which would double-count the reduced distance).
            let path_cost = dist_red[sink] + h[sink] - h[source];

            // Update potentials for every vertex reachable in the residual graph.
            let mut i: usize = 0;
            while i < n {
                let di = dist_red[i];
                if di < INF {
                    let new_h = h[i] + di;
                    h[i] = new_h;
                }
                i += 1;
            }

            // Bottleneck = minimum residual capacity along the source→sink path.
            let mut bottleneck: i64 = 0;
            let mut b_set: bool = false;
            let mut v: usize = sink;
            while v != source {
                let u = parent[v];
                if parent_fwd[v] {
                    let cap_uv = cap[idx(u, n, v)];
                    let flow_uv = flow[idx(u, n, v)];
                    let rcap = cap_uv - flow_uv;
                    if !b_set {
                        bottleneck = rcap;
                        b_set = true;
                    } else {
                        if rcap < bottleneck {
                            bottleneck = rcap;
                        }
                    }
                } else {
                    let rcap = flow[idx(v, n, u)];
                    if !b_set {
                        bottleneck = rcap;
                        b_set = true;
                    } else {
                        if rcap < bottleneck {
                            bottleneck = rcap;
                        }
                    }
                }
                v = u;
            }

            // Augment `bottleneck` units along the path.
            let mut v2: usize = sink;
            while v2 != source {
                let u = parent[v2];
                if parent_fwd[v2] {
                    let i_fwd = idx(u, n, v2);
                    let new_flow = flow[i_fwd] + bottleneck;
                    flow[i_fwd] = new_flow;
                } else {
                    let i_rev = idx(v2, n, u);
                    let new_flow = flow[i_rev] - bottleneck;
                    flow[i_rev] = new_flow;
                }
                v2 = u;
            }

            total_flow = total_flow + bottleneck;
            let cost_add = bottleneck * path_cost;
            total_cost = total_cost + cost_add;
        }
    }

    Ok((total_flow, total_cost, flow))
}
