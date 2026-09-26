//! Floyd–Warshall all-pairs shortest path (dense matrix, O(V³) dynamic
//! programming).  Supports negative edge weights and detects negative cycles.
//!
//! Distances are `Option<i64>`: `None` means "no path" (infinite distance),
//! `Some(d)` means a shortest distance of `d`.  This reserves no sentinel
//! value in the integer domain, so every `i64` is a valid weight.

/// Compute all-pairs shortest distances via Floyd–Warshall.
///
/// `weight` is the flattened row-major `n × n` adjacency matrix:
/// `weight[i*n + j]` is `Some(w)` for an edge `i → j` of weight `w`, or
/// `None` for a missing edge.  The diagonal `weight[i*n + i]` must be
/// `Some(0)`.
///
/// Returns `Some(dist)` where `dist[i*n + j]` is `Some(d)` with `d` the
/// shortest distance from `i` to `j`, or `None` when `j` is unreachable from
/// `i`.  The outer `None` is returned when a negative-weight cycle is present.
pub fn floyd_warshall(n: usize, weight: &[Option<i64>]) -> Option<Vec<Option<i64>>> {
    // Copy the input matrix element-by-element (avoids `Option`'s noncomputable
    // `Clone` instance that `weight.to_vec()` would pull in).
    let mut dist: Vec<Option<i64>> = Vec::new();
    let nn = n * n;
    let mut idx: usize = 0;
    while idx < nn {
        let w = weight[idx];
        dist.push(w);
        idx += 1;
    }

    let mut k: usize = 0;
    while k < n {
        let mut i: usize = 0;
        while i < n {
            let mut j: usize = 0;
            while j < n {
                let ik = i * n + k;
                let kj = k * n + j;
                let dik = dist[ik];
                let dkj = dist[kj];
                match dik {
                    None => {}
                    Some(a) => {
                        match dkj {
                            None => {}
                            Some(b) => {
                                let cand = a + b;
                                let ij = i * n + j;
                                let dij = dist[ij];
                                match dij {
                                    None => {
                                        dist[ij] = Some(cand);
                                    }
                                    Some(c) => {
                                        if cand < c {
                                            dist[ij] = Some(cand);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                j += 1;
            }
            i += 1;
        }
        k += 1;
    }

    // A negative diagonal entry means a negative-weight cycle.
    let mut has_neg_cycle: bool = false;
    let mut i: usize = 0;
    while i < n {
        let ii = i * n + i;
        let dii = dist[ii];
        match dii {
            None => {}
            Some(d) => {
                if d < 0 {
                    has_neg_cycle = true;
                }
            }
        }
        i += 1;
    }

    if has_neg_cycle {
        None
    } else {
        Some(dist)
    }
}
