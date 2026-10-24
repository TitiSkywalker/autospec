//! Stable marriage via the Gale–Shapley (deferred acceptance) algorithm.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Compute a stable matching of `n` men and `n` women via the men-proposing
/// Gale–Shapley algorithm.
///
/// `rank_m` and `rank_w` are flat, row-major `n × n` rank matrices:
/// `rank_m[idx(m, n, w)]` is the rank of woman `w` in man `m`'s preference
/// order (`0` = most preferred, smaller is better), and symmetrically for
/// `rank_w`.  Preferences are strict and complete (each row is a permutation
/// of `0 .. n`).
///
/// Returns a flat `n × n` 0/1 matrix `matched` with `matched[u,v] == 1` when
/// man `u` is matched to woman `v`.  The matching is stable: no man–woman pair
/// both prefer each other over their assigned partner.
pub fn stable_match(n: usize, rank_m: &[usize], rank_w: &[usize]) -> Vec<usize> {
    // match_m[m] = woman matched to man m (`n` = single).
    // match_w[w] = man matched to woman w (`n` = single).
    let mut match_m: Vec<usize> = vec![n; n];
    let mut match_w: Vec<usize> = vec![n; n];
    // next[m] = rank of the next woman man m will propose to.
    let mut next: Vec<usize> = vec![0usize; n];

    let mut progress: bool = true;
    while progress {
        progress = false;
        let mut m: usize = 0;
        while m < n {
            if match_m[m] == n && next[m] < n {
                progress = true;
                let nr = next[m];
                // The woman man m proposes to: the one of rank `nr`.
                let mut w: usize = n;
                let mut v: usize = 0;
                while v < n {
                    if rank_m[idx(m, n, v)] == nr {
                        w = v;
                        v = n;
                    }
                    v += 1;
                }
                let nn = next[m] + 1;
                next[m] = nn;
                if match_w[w] == n {
                    match_m[m] = w;
                    match_w[w] = m;
                } else {
                    let cur = match_w[w];
                    let rm = rank_w[idx(w, n, m)];
                    let rc = rank_w[idx(w, n, cur)];
                    if rm < rc {
                        match_m[cur] = n;
                        match_m[m] = w;
                        match_w[w] = m;
                    }
                }
            }
            m += 1;
        }
    }

    let mut matched: Vec<usize> = vec![0usize; n * n];
    let mut m2: usize = 0;
    while m2 < n {
        if match_m[m2] != n {
            let i = idx(m2, n, match_m[m2]);
            matched[i] = 1;
        }
        m2 += 1;
    }
    matched
}
