//! Maximum bipartite matching via Hopcroft–Karp's algorithm.

/// Flat index for a row-major `rows × cols` matrix.
fn idx(row: usize, cols: usize, col: usize) -> usize {
    let t = row * cols;
    let r = t + col;
    r
}

/// BFS from all free left vertices, computing the shortest-augmenting-path
/// levels `dist_l` / `dist_r` and the length `shortest` of a shortest
/// augmenting path (`-1` when none exists).
fn bfs_levels(
    nl: usize,
    nr: usize,
    adj: &[i64],
    match_l: &[usize],
    match_r: &[usize],
) -> (Vec<i64>, Vec<i64>, i64) {
    let mut dist_l: Vec<i64> = vec![-1i64; nl];
    let mut dist_r: Vec<i64> = vec![-1i64; nr];
    let mut queue: Vec<usize> = vec![0usize; nl];
    let mut head: usize = 0;
    let mut tail: usize = 0;
    let mut u: usize = 0;
    while u < nl {
        if match_l[u] == nr {
            dist_l[u] = 0;
            queue[tail] = u;
            tail += 1;
        }
        u += 1;
    }
    let mut shortest: i64 = -1;
    while head < tail {
        let u = queue[head];
        head += 1;
        let lu = dist_l[u];
        let mut v: usize = 0;
        while v < nr {
            if adj[idx(u, nr, v)] == 1 && v != match_l[u] && dist_r[v] == -1 {
                let new_dr = lu + 1;
                dist_r[v] = new_dr;
                if match_r[v] == nl {
                    if shortest == -1 {
                        shortest = new_dr;
                    }
                } else {
                    let u2 = match_r[v];
                    if dist_l[u2] == -1 {
                        let new_dl = new_dr + 1;
                        dist_l[u2] = new_dl;
                        queue[tail] = u2;
                        tail += 1;
                    }
                }
            }
            v += 1;
        }
    }
    (dist_l, dist_r, shortest)
}

/// DFS phase: augment along shortest augmenting paths until none remains in
/// the current level graph.  Consumes and returns the matching arrays.
fn dfs_phase(
    nl: usize,
    nr: usize,
    adj: &[i64],
    mut match_l: Vec<usize>,
    mut match_r: Vec<usize>,
    dist_l: &[i64],
    dist_r: &[i64],
    shortest: i64,
) -> (Vec<usize>, Vec<usize>) {
    let mut dead_l: Vec<bool> = vec![false; nl];
    let mut dead_r: Vec<bool> = vec![false; nr];
    let mut any_aug: bool = true;
    while any_aug {
        any_aug = false;
        let mut src: usize = 0;
        while src < nl {
            if match_l[src] == nr && dist_l[src] == 0 && !dead_l[src] {
                // Iterative DFS for one augmenting path from `src`.  The stack
                // alternates left/right: even entries are left vertices (raw
                // index), odd entries are right vertices (encoded as `nl + v`).
                let mut st: Vec<usize> = vec![0usize; nl + nr];
                let mut st_len: usize = 0;
                st[0] = src;
                st_len = 1;
                let mut found: bool = false;
                while st_len > 0 && !found {
                    let u = st[st_len - 1];
                    let lu = dist_l[u];
                    let want = lu + 1;
                    let mut next_v: usize = nr;
                    if want <= shortest {
                        let mut v: usize = 0;
                        while v < nr {
                            if adj[idx(u, nr, v)] == 1 && v != match_l[u] &&
                               !dead_r[v] && dist_r[v] == want {
                                let mv = match_r[v];
                                if mv == nl {
                                    next_v = v;
                                    v = nr;
                                } else {
                                    let dl_mv = dist_l[mv];
                                    let want2 = want + 1;
                                    if dl_mv == want2 {
                                        next_v = v;
                                        v = nr;
                                    }
                                }
                            }
                            v += 1;
                        }
                    }
                    if next_v != nr {
                        let enc = nl + next_v;
                        st[st_len] = enc;
                        st_len += 1;
                        if match_r[next_v] == nl {
                            found = true;
                        } else {
                            let u2 = match_r[next_v];
                            st[st_len] = u2;
                            st_len += 1;
                        }
                    } else {
                        // Dead end: backtrack past `u` and the right vertex
                        // that led to it.
                        dead_l[u] = true;
                        st_len -= 1;
                        if st_len >= 1 {
                            let rc = st[st_len - 1];
                            let rv = rc - nl;
                            dead_r[rv] = true;
                            st_len -= 1;
                        }
                    }
                }
                if found {
                    // Flip the matching along the path: st[2i] — st[2i+1].
                    let mut i: usize = 0;
                    while i + 1 < st_len {
                        let left = st[i];
                        let right = st[i + 1] - nl;
                        match_l[left] = right;
                        match_r[right] = left;
                        i += 2;
                    }
                    any_aug = true;
                }
            }
            src += 1;
        }
    }
    (match_l, match_r)
}

/// Compute a maximum matching of a bipartite graph via Hopcroft–Karp.
///
/// The graph has `nl` left vertices and `nr` right vertices.  `adj` is a flat,
/// row-major `nl × nr` 0/1 matrix: `adj[idx(u, nr, v)] == 1` means left vertex
/// `u` is adjacent to right vertex `v`.
///
/// Returns `(size, matched)` where `size` is the cardinality of a maximum
/// matching and `matched` is a flat `nl × nr` 0/1 matrix with
/// `matched[u,v] == 1` exactly on the matched edges.
pub fn hopcroft_karp(nl: usize, nr: usize, adj: &[i64]) -> (i64, Vec<i64>) {
    // match_l[u] = right vertex matched to left `u` (`nr` = unmatched).
    // match_r[v] = left vertex matched to right `v` (`nl` = unmatched).
    let mut match_l: Vec<usize> = vec![nr; nl];
    let mut match_r: Vec<usize> = vec![nl; nr];

    let mut progress: bool = true;
    while progress {
        let (dist_l, dist_r, shortest) = bfs_levels(nl, nr, adj, &match_l, &match_r);
        if shortest == -1 {
            // No augmenting path: the matching is maximum (Berge's theorem).
            progress = false;
        } else {
            let (ml2, mr2) = dfs_phase(nl, nr, adj, match_l, match_r, &dist_l, &dist_r, shortest);
            match_l = ml2;
            match_r = mr2;
        }
    }

    // Build the 0/1 matching matrix and its cardinality.
    let mut matched: Vec<i64> = vec![0i64; nl * nr];
    let mut size: i64 = 0;
    let mut u: usize = 0;
    while u < nl {
        if match_l[u] != nr {
            let i = idx(u, nr, match_l[u]);
            matched[i] = 1;
            size = size + 1;
        }
        u += 1;
    }
    (size, matched)
}
