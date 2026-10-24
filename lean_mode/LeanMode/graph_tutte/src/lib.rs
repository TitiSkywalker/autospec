//! Tutte's 1-factor condition, checked by brute-force enumeration of all `2^n`
//! vertex subsets.
//!
//! The graph is given as a flat, row-major `n × n` 0/1 *symmetric* adjacency
//! matrix: `adj[u * n + v] == 1` iff the undirected edge `{u, v}` is present.
//!
//! Tutte's theorem (1947) states that a finite undirected graph `G` has a
//! perfect matching **iff** for every `S ⊆ V` the number of *odd* connected
//! components of `G − S` is at most `|S|`.  This program decides that
//! condition (its decision problem is the hypothesis side of the theorem).

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// `2 ^ e`, computed by a doubling `while` loop (no shifts, per the translator
/// constraints).
fn pow2(e: usize) -> usize {
    let mut p: usize = 1;
    let mut i: usize = 0;
    while i < e {
        p = p * 2;
        i += 1;
    }
    p
}

/// Bit `i` of `mask` (`0` or `1`), computed with `/` and `%` only.
fn bit(mask: usize, i: usize) -> usize {
    let p = pow2(i);
    let q = mask / p;
    let b = q % 2;
    b
}

/// Population count of `mask`, via a halving `while` loop.
fn popcount(mut x: usize) -> usize {
    let mut c: usize = 0;
    while x > 0 {
        let b = x % 2;
        let t = c + b;
        c = t;
        x = x / 2;
    }
    c
}

/// Decide Tutte's 1-factor condition: for every `S ⊆ V`, the number of odd
/// connected components of `G − S` is at most `|S|`.
///
/// Enumerates all `2^n` subsets `S` as bitmasks.  For each `S` it removes the
/// vertices of `S`, counts the connected components of the induced graph on
/// `V \ S` by BFS (a `usize` queue + a `visited` array, as in
/// `graph_bipartite_check`), and checks `odd_count ≤ popcount(S)`.
pub fn tutte_condition_holds(n: usize, adj: &[usize]) -> bool {
    let limit = pow2(n);

    // BFS scratch space, reused across components and across subsets.
    let mut queue: Vec<usize> = vec![0usize; n];

    let mut ok: bool = true;
    let mut mask: usize = 0;
    while mask < limit {
        // visited[v] == 1 iff v ∈ S, or v has already been put in a component.
        let mut visited: Vec<usize> = vec![0usize; n];
        let mut i: usize = 0;
        while i < n {
            let b = bit(mask, i);
            if b == 1 {
                visited[i] = 1;
            }
            i += 1;
        }

        let mut odd_count: usize = 0;
        let mut s: usize = 0;
        while s < n {
            if visited[s] == 0 {
                // New component of G − S; BFS it.
                visited[s] = 1;
                let mut head: usize = 0;
                let mut tail: usize = 0;
                queue[tail] = s;
                tail += 1;
                let mut comp_size: usize = 0;

                while head < tail {
                    let u = queue[head];
                    head += 1;
                    comp_size += 1;

                    let mut v: usize = 0;
                    while v < n {
                        if visited[v] == 0 {
                            let a = adj[idx(u, n, v)];
                            if a == 1 {
                                visited[v] = 1;
                                queue[tail] = v;
                                tail += 1;
                            }
                        }
                        v += 1;
                    }
                }

                let par = comp_size % 2;
                if par == 1 {
                    odd_count += 1;
                }
            }
            s += 1;
        }

        let pc = popcount(mask);
        if odd_count > pc {
            ok = false;
        }

        mask += 1;
    }

    ok
}
