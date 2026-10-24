//! Global minimum cut of an undirected weighted graph via Stoer–Wagner's
//! algorithm.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Compute the value of a global minimum cut of an undirected weighted graph
/// using Stoer–Wagner's algorithm.
///
/// The graph is a flat, row-major, symmetric `n × n` weight matrix:
/// `weight[idx(u, n, v)]` is the weight of the undirected edge `u — v`
/// (`0` = no edge, diagonal ignored).  Weights must be non-negative.
///
/// Returns the total weight of a minimum cut (a non-empty proper vertex subset
/// minimizing the total weight of its crossing edges).  `n` must be at least 2.
pub fn stoer_wagner(n: usize, weight: &[i64]) -> i64 {
    let mut w: Vec<i64> = weight.to_vec();
    let mut active: Vec<bool> = vec![true; n];
    let mut active_count: usize = n;
    let mut best: i64 = 0;
    let mut best_set: bool = false;

    while active_count > 1 {
        // One phase = a maximum-adjacency search: grow a set A one vertex at a
        // time, always adding the vertex most tightly connected to A.
        let mut added: Vec<bool> = vec![false; n];
        let mut w_to_a: Vec<i64> = vec![0i64; n];

        // Start the phase at the first still-active vertex.
        let mut first: usize = n;
        let mut i: usize = 0;
        while i < n {
            if active[i] {
                first = i;
                i = n;
            } else {
                i += 1;
            }
        }
        added[first] = true;
        let mut prev: usize = first;
        let mut s: usize = first;
        let mut t: usize = first;
        let mut added_count: usize = 1;

        while added_count < active_count {
            // Raise the connectivity of every candidate by w(prev, candidate).
            let mut u: usize = 0;
            while u < n {
                if active[u] && !added[u] {
                    let w_prev_u = w[idx(prev, n, u)];
                    let new_w = w_to_a[u] + w_prev_u;
                    w_to_a[u] = new_w;
                }
                u += 1;
            }
            // Pick the candidate with the largest connectivity to A.
            let mut pick: usize = n;
            let mut max_w: i64 = 0;
            let mut found: bool = false;
            let mut u2: usize = 0;
            while u2 < n {
                if active[u2] && !added[u2] {
                    let wu = w_to_a[u2];
                    if !found {
                        max_w = wu;
                        pick = u2;
                        found = true;
                    } else {
                        if wu > max_w {
                            max_w = wu;
                            pick = u2;
                        }
                    }
                }
                u2 += 1;
            }
            added[pick] = true;
            s = prev;
            t = pick;
            prev = pick;
            added_count += 1;
        }

        // The cut of the phase separates t from A; its value is w_to_a[t].
        // Seed `best` with the first phase cut, then take the minimum (every
        // phase cut is a real cut, so no sentinel value is needed).
        let cut = w_to_a[t];
        if !best_set {
            best = cut;
            best_set = true;
        } else {
            if cut < best {
                best = cut;
            }
        }

        // Merge t into s: fold t's edges into s and deactivate t.
        let mut u3: usize = 0;
        while u3 < n {
            if active[u3] && u3 != s && u3 != t {
                let w_s_u = w[idx(s, n, u3)];
                let w_t_u = w[idx(t, n, u3)];
                let new_su = w_s_u + w_t_u;
                w[idx(s, n, u3)] = new_su;
                w[idx(u3, n, s)] = new_su;
            }
            u3 += 1;
        }
        active[t] = false;
        active_count = active_count - 1;
    }

    best
}
