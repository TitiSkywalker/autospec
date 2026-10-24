//! Maximum flow (Dinic's algorithm) on a directed graph given as a flat
//! capacity matrix.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Compute a maximum `source`–`sink` flow via Dinic's algorithm
/// (level graph + blocking flow).
///
/// The graph is a directed capacity matrix: `capacity` is a flat `n × n`
/// row-major slice where `capacity[idx(u, n, v)]` is the capacity of the
/// directed edge `u → v` (0 means "no edge").
///
/// Returns `(value, flow)` where `value` is the max-flow value and `flow` is
/// a skew-symmetric flat `n × n` flow matrix: `flow[u,v] ≤ capacity[u,v]`,
/// `flow[v,u] = -flow[u,v]`, and flow is conserved at every vertex except
/// `source` and `sink`.
pub fn dinic(
    n: usize,
    capacity: &[i64],
    source: usize,
    sink: usize,
) -> (i64, Vec<i64>) {
    let mut flow: Vec<i64> = vec![0i64; n * n];

    let mut progress: bool = source != sink;
    while progress {
        // BFS: compute shortest-path levels in the residual graph
        // (`level[v] == -1` means "not yet reached").
        let mut level: Vec<i64> = vec![-1i64; n];
        level[source] = 0;
        let mut queue: Vec<usize> = vec![0usize; n];
        queue[0] = source;
        let mut head: usize = 0;
        let mut tail: usize = 1;
        while head < tail {
            let u = queue[head];
            head += 1;
            let mut v: usize = 0;
            while v < n {
                let cap = capacity[idx(u, n, v)];
                let f = flow[idx(u, n, v)];
                let residual = cap - f;
                if residual > 0 && level[v] == -1 {
                    let lu = level[u];
                    let new_level = lu + 1;
                    level[v] = new_level;
                    queue[tail] = v;
                    tail += 1;
                }
                v += 1;
            }
        }

        if level[sink] == -1 {
            // Sink unreachable in the residual graph: the flow is maximum.
            progress = false;
        } else {
            // Blocking flow: repeatedly find and augment an s→t path whose
            // levels strictly increase by 1, until none remains.
            let mut dead: Vec<bool> = vec![false; n];
            let mut searching: bool = true;
            while searching {
                let mut stack: Vec<usize> = vec![0usize; n];
                let mut on_path: Vec<bool> = vec![false; n];
                let mut stack_len: usize = 0;
                stack[0] = source;
                stack_len = 1;
                on_path[source] = true;

                // Iterative DFS for one level-increasing s→t path.
                let mut found: bool = false;
                while stack_len > 0 && !found {
                    let u = stack[stack_len - 1];
                    if u == sink {
                        found = true;
                    } else {
                        let lu = level[u];
                        let want = lu + 1;
                        let mut next: usize = n;
                        let mut v: usize = 0;
                        while v < n {
                            if !on_path[v] && !dead[v] && level[v] == want {
                                let cap = capacity[idx(u, n, v)];
                                let f = flow[idx(u, n, v)];
                                let residual = cap - f;
                                if residual > 0 {
                                    next = v;
                                    v = n;
                                }
                            }
                            v += 1;
                        }
                        if next != n {
                            stack[stack_len] = next;
                            stack_len += 1;
                            on_path[next] = true;
                        } else {
                            stack_len -= 1;
                            on_path[u] = false;
                            dead[u] = true;
                        }
                    }
                }

                if found {
                    // Bottleneck = minimum residual capacity along the path.
                    let mut bottleneck: i64 = 0;
                    let mut b_set: bool = false;
                    let mut k: usize = 0;
                    while k + 1 < stack_len {
                        let u = stack[k];
                        let v = stack[k + 1];
                        let cap = capacity[idx(u, n, v)];
                        let f = flow[idx(u, n, v)];
                        let residual = cap - f;
                        if !b_set {
                            bottleneck = residual;
                            b_set = true;
                        } else {
                            if residual < bottleneck {
                                bottleneck = residual;
                            }
                        }
                        k += 1;
                    }

                    // Augment `bottleneck` units along the path (forward +
                    // reverse, keeping the flow matrix skew-symmetric).
                    let mut k2: usize = 0;
                    while k2 + 1 < stack_len {
                        let u = stack[k2];
                        let v = stack[k2 + 1];
                        let i_fwd = idx(u, n, v);
                        let i_rev = idx(v, n, u);
                        let new_fwd = flow[i_fwd] + bottleneck;
                        flow[i_fwd] = new_fwd;
                        let new_rev = flow[i_rev] - bottleneck;
                        flow[i_rev] = new_rev;
                        k2 += 1;
                    }
                } else {
                    searching = false;
                }
            }
        }
    }

    // Flow value = net outflow from `source`.
    let mut value: i64 = 0;
    let mut v: usize = 0;
    while v < n {
        let f = flow[idx(source, n, v)];
        value = value + f;
        v += 1;
    }

    (value, flow)
}
