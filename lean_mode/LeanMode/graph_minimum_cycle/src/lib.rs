//! Minimum cycle (girth) of an undirected weighted graph via the
//! Floyd–Warshall "split by the largest vertex" trick.

/// "Infinity" sentinel for absent edges / unreachable distances.  Large enough
/// to be unattainable by any real cycle, small enough that finite sums cannot
/// overflow `i64` for the modest weights in the test suite.
const INF: i64 = 1_000_000_000_000_000_000;

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Minimum cycle of an undirected weighted graph (Floyd variant).
///
/// `weight` is a flat, row-major, symmetric `n × n` matrix with
/// `weight[u,v]` the weight of the undirected edge `u — v` (`INF` = no edge),
/// non-negative weights, and a zero diagonal.
///
/// Returns `Ok(min)` where `min` is the minimum total weight of a simple cycle
/// with at least three distinct vertices, or `Err(())` when the graph has no
/// cycle.  Every simple cycle `u₁ → … → uₘ → u₁` is "split" at its
/// largest-indexed vertex `k` into the two edges `(i, k)`, `(k, j)` and a path
/// `j ⇝ i` using only vertices `< k`; enumerating that split is the subtle part.
pub fn graph_minimum_cycle(n: usize, weight: &[i64]) -> Result<i64, ()> {
    let mut dis: Vec<i64> = weight.to_vec();
    let mut best: i64 = 0;
    let mut found: bool = false;

    let mut k: usize = 0;
    while k < n {
        // Before relaxing through `k`, every cycle whose largest vertex is `k`
        // is `dis[i][j] + weight[i][k] + weight[k][j]` for some `j < i < k`.
        let mut i: usize = 0;
        while i < k {
            let mut j: usize = 0;
            while j < i {
                let gik = weight[idx(i, n, k)];
                let gkj = weight[idx(k, n, j)];
                let dij = dis[idx(i, n, j)];
                if gik < INF && gkj < INF && dij < INF {
                    let cand1 = gik + gkj;
                    let cand = cand1 + dij;
                    if !found {
                        best = cand;
                        found = true;
                    } else {
                        if cand < best {
                            best = cand;
                        }
                    }
                }
                j += 1;
            }
            i += 1;
        }

        // Standard Floyd–Warshall relaxation through `k`.
        let mut i2: usize = 0;
        while i2 < n {
            let mut j2: usize = 0;
            while j2 < n {
                let dik = dis[idx(i2, n, k)];
                let dkj = dis[idx(k, n, j2)];
                if dik < INF && dkj < INF {
                    let nd = dik + dkj;
                    let dij = dis[idx(i2, n, j2)];
                    if nd < dij {
                        dis[idx(i2, n, j2)] = nd;
                    }
                }
                j2 += 1;
            }
            i2 += 1;
        }

        k += 1;
    }

    if found {
        Ok(best)
    } else {
        Err(())
    }
}
