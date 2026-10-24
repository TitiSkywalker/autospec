//! Negative-cycle detection with a feasible-potential certificate (Bellman–Ford
//! from a virtual super-source).

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Detect a negative-cost cycle and produce a feasible potential when none
/// exists.
///
/// The directed graph is given by two flat, row-major `n × n` matrices: `cap`
/// (edge capacity, `0` = no edge) and `cost` (per-unit edge cost, which may be
/// negative).  Edge `u → v` exists iff `cap[u,v] > 0`.
///
/// Returns `(has_neg_cycle, h)` where:
/// - `has_neg_cycle = true` iff the graph has a directed cycle whose total cost
///   is negative (Bellman–Ford keeps relaxing forever);
/// - otherwise `h` is a *feasible potential*: every edge satisfies the reduced
///   cost `cost[u,v] + h[u] - h[v] ≥ 0`.
///
/// The equivalence "no negative cycle iff a feasible potential exists" is the
/// LP-duality / potential argument (Johnson's technique in reverse).
pub fn negative_cycle(n: usize, cap: &[i64], cost: &[i64]) -> (bool, Vec<i64>) {
    let mut h: Vec<i64> = vec![0i64; n];

    // Bellman–Ford from a virtual super-source (every vertex has potential 0).
    let mut converged: bool = false;
    let mut pass: usize = 0;
    while pass < n && !converged {
        let mut changed: bool = false;
        let mut u: usize = 0;
        while u < n {
            let mut v: usize = 0;
            while v < n {
                let c = cap[idx(u, n, v)];
                if c > 0 {
                    let hu = h[u];
                    let cuv = cost[idx(u, n, v)];
                    let new_h = hu + cuv;
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

    // One more relaxation pass: a still-relaxable edge means a negative cycle.
    let mut has_neg: bool = false;
    let mut u2: usize = 0;
    while u2 < n {
        let mut v2: usize = 0;
        while v2 < n {
            let c = cap[idx(u2, n, v2)];
            if c > 0 {
                let hu = h[u2];
                let cuv = cost[idx(u2, n, v2)];
                let new_h = hu + cuv;
                if new_h < h[v2] {
                    has_neg = true;
                }
            }
            v2 += 1;
        }
        u2 += 1;
    }

    (has_neg, h)
}
