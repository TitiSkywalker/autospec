//! Feasible circulation with lower bounds (auxiliary source–sink
//! transformation + max flow).

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Maximum `source`–`sink` flow (Edmonds–Karp) on a directed graph given as a
/// flat `n × n` capacity matrix.  Returns a non-negative flat `n × n` flow
/// matrix: `flow[u,v]` is the flow sent on the directed edge `u → v`.
fn max_flow(n: usize, cap: &[i64], source: usize, sink: usize) -> Vec<i64> {
    let mut flow: Vec<i64> = vec![0i64; n * n];

    let mut progress: bool = source != sink;
    while progress {
        // BFS over residual edges (forward `cap - flow`, reverse `flow`).
        let mut parent: Vec<usize> = vec![n; n];
        let mut parent_fwd: Vec<bool> = vec![false; n];
        let mut queue: Vec<usize> = vec![0usize; n];
        parent[source] = source;
        queue[0] = source;
        let mut head: usize = 0;
        let mut tail: usize = 1;
        while head < tail {
            let u = queue[head];
            head += 1;
            let mut v: usize = 0;
            while v < n {
                if parent[v] == n {
                    let cap_uv = cap[idx(u, n, v)];
                    let flow_uv = flow[idx(u, n, v)];
                    let r_f = cap_uv - flow_uv;
                    if r_f > 0 {
                        parent[v] = u;
                        parent_fwd[v] = true;
                        queue[tail] = v;
                        tail += 1;
                    }
                }
                if parent[v] == n {
                    let flow_vu = flow[idx(v, n, u)];
                    if flow_vu > 0 {
                        parent[v] = u;
                        parent_fwd[v] = false;
                        queue[tail] = v;
                        tail += 1;
                    }
                }
                v += 1;
            }
        }

        if parent[sink] == n {
            progress = false;
        } else {
            // Bottleneck = minimum residual capacity along the path.
            let mut bottleneck: i64 = 0;
            let mut b_set: bool = false;
            let mut v: usize = sink;
            while v != source {
                let u = parent[v];
                if parent_fwd[v] {
                    let cap_uv = cap[idx(u, n, v)];
                    let flow_uv = flow[idx(u, n, v)];
                    let r = cap_uv - flow_uv;
                    if !b_set {
                        bottleneck = r;
                        b_set = true;
                    } else {
                        if r < bottleneck {
                            bottleneck = r;
                        }
                    }
                } else {
                    let r = flow[idx(v, n, u)];
                    if !b_set {
                        bottleneck = r;
                        b_set = true;
                    } else {
                        if r < bottleneck {
                            bottleneck = r;
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
                    let i_f = idx(u, n, v2);
                    let new_f = flow[i_f] + bottleneck;
                    flow[i_f] = new_f;
                } else {
                    let i_r = idx(v2, n, u);
                    let new_r = flow[i_r] - bottleneck;
                    flow[i_r] = new_r;
                }
                v2 = u;
            }
        }
    }

    flow
}

/// Find a feasible circulation with lower bounds on a directed graph.
///
/// `low` and `cap` are flat, row-major `n × n` matrices: the directed edge
/// `u → v` must carry flow between `low[u,v]` and `cap[u,v]` (0 = no edge,
/// `low[u,v] ≤ cap[u,v]`).  A *circulation* conserves flow at every vertex
/// (inflow equals outflow).
///
/// Returns `Ok(flow)` with `low[u,v] ≤ flow[u,v] ≤ cap[u,v]` and conservation
/// at every vertex, or `Err(())` when no feasible circulation exists.
pub fn flow_bound(n: usize, low: &[i64], cap: &[i64]) -> Result<Vec<i64>, ()> {
    // Initial flow = lower bounds.  `bal[v]` is its imbalance:
    // `bal[v] = inflow_low[v] - outflow_low[v]`.
    let mut bal: Vec<i64> = vec![0i64; n];
    let mut u: usize = 0;
    while u < n {
        let mut v: usize = 0;
        while v < n {
            let l = low[idx(u, n, v)];
            let bal_u = bal[u] - l;
            bal[u] = bal_u;
            let bal_v = bal[v] + l;
            bal[v] = bal_v;
            v += 1;
        }
        u += 1;
    }

    // Auxiliary network on `n + 2` vertices: super-source `n`, super-sink `n+1`.
    let m = n + 2;
    let ss = n;
    let tt = n + 1;
    let mut rcap: Vec<i64> = vec![0i64; m * m];
    let mut u2: usize = 0;
    while u2 < n {
        let mut v: usize = 0;
        while v < n {
            let c = cap[idx(u2, n, v)];
            let l = low[idx(u2, n, v)];
            let r = c - l;
            rcap[idx(u2, m, v)] = r;
            v += 1;
        }
        u2 += 1;
    }
    let mut total: i64 = 0;
    let mut w: usize = 0;
    while w < n {
        let b = bal[w];
        if b > 0 {
            rcap[idx(ss, m, w)] = b;
            total = total + b;
        } else {
            let nb = 0 - b;
            rcap[idx(w, m, tt)] = nb;
        }
        w += 1;
    }

    let rflow = max_flow(m, &rcap, ss, tt);

    // Saturation check: the super-source must send all `total` units.
    let mut sent: i64 = 0;
    let mut w2: usize = 0;
    while w2 < n {
        let f = rflow[idx(ss, m, w2)];
        sent = sent + f;
        w2 += 1;
    }
    if sent < total {
        return Err(());
    }

    // Feasible flow = lower bound + residual flow on each original edge.
    let mut flow: Vec<i64> = vec![0i64; n * n];
    let mut u3: usize = 0;
    while u3 < n {
        let mut v: usize = 0;
        while v < n {
            let l = low[idx(u3, n, v)];
            let rf = rflow[idx(u3, m, v)];
            let f = l + rf;
            flow[idx(u3, n, v)] = f;
            v += 1;
        }
        u3 += 1;
    }
    Ok(flow)
}
