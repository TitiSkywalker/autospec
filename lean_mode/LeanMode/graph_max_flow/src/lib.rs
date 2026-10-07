//! Maximum flow (Edmonds-Karp) on a directed graph given as a flat
//! capacity matrix.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Compute a maximum `source`–`sink` flow via the Edmonds-Karp algorithm
/// (Ford-Fulkerson with BFS shortest augmenting paths).
///
/// The graph is a directed capacity matrix: `capacity` is a flat `n × n`
/// row-major slice where `capacity[idx(u, n, v)]` is the capacity of the
/// directed edge `u → v` (0 means "no edge").
///
/// Returns `(value, flow)` where `value` is the max-flow value and `flow` is
/// a skew-symmetric flat `n × n` flow matrix: `flow[u,v] ≤ capacity[u,v]`,
/// `flow[v,u] = -flow[u,v]`, and flow is conserved at every vertex except
/// `source` and `sink`.
pub fn max_flow(
    n: usize,
    capacity: &[i64],
    source: usize,
    sink: usize,
) -> (i64, Vec<i64>) {
    let mut flow: Vec<i64> = vec![0i64; n * n];

    // `parent[v] == n` means "not yet discovered" during a BFS.
    let mut progress: bool = source != sink;
    while progress {
        let mut parent: Vec<usize> = vec![n; n];
        let mut queue: Vec<usize> = vec![0usize; n];

        parent[source] = source;
        queue[0] = source;
        let mut head: usize = 0;
        let mut tail: usize = 1;

        // BFS from `source` over residual edges (residual capacity > 0).
        while head < tail {
            let u = queue[head];
            head += 1;

            let mut v: usize = 0;
            while v < n {
                if parent[v] == n {
                    let cap = capacity[idx(u, n, v)];
                    let f = flow[idx(u, n, v)];
                    let residual = cap - f;
                    if residual > 0 {
                        parent[v] = u;
                        queue[tail] = v;
                        tail += 1;
                    }
                }
                v += 1;
            }
        }

        if parent[sink] == n {
            // No augmenting path: the flow is maximum.
            progress = false;
        } else {
            // Bottleneck = minimum residual capacity along the path.
            let mut b: i64 = 0;
            let mut b_set: bool = false;
            let mut v: usize = sink;
            while v != source {
                let u = parent[v];
                let cap = capacity[idx(u, n, v)];
                let f = flow[idx(u, n, v)];
                let residual = cap - f;
                if !b_set {
                    b = residual;
                    b_set = true;
                } else if residual < b {
                    b = residual;
                }
                v = u;
            }

            // Augment `b` units along the path (forward + reverse).
            let mut v2: usize = sink;
            while v2 != source {
                let u = parent[v2];
                let i_fwd = idx(u, n, v2);
                let i_rev = idx(v2, n, u);
                let new_fwd = flow[i_fwd] + b;
                flow[i_fwd] = new_fwd;
                let new_rev = flow[i_rev] - b;
                flow[i_rev] = new_rev;
                v2 = u;
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
