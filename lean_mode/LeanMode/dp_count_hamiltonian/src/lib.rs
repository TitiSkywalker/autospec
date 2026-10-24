//! Counting Hamiltonian paths by inclusion–exclusion (a Ryser-style sum).
//!
//! `adj` is the flat row-major `n × n` adjacency matrix of a digraph: the entry
//! `adj[u * n + v]` is `1` when the edge `u → v` exists and `0` otherwise.
//!
//! A Hamiltonian path is an ordering `v₀, …, v_{n-1}` of **all** `n` vertices in
//! which every consecutive pair `vᵢ → vᵢ₊₁` is an edge.  Writing `W(S)` for the
//! number of vertex sequences of length `n` that only use vertices of `S` and
//! follow edges, inclusion–exclusion over the vertex sets gives
//!
//! ```text
//! #{Hamiltonian paths} = Σ_{S ⊆ V} (−1)^{n − |S|} · W(S) ,
//! ```
//!
//! because the sequences counted by `W(V)` whose *support* is all of `V` are
//! exactly the Hamiltonian paths, and a length-`n` sequence over `n` vertices
//! with full support uses every vertex exactly once.
//!
//! `W(S)` is computed with one DP sweep: `w[v]` starts at `1` for `v ∈ S` and
//! every further step replaces it by `Σ_u w[u] · adj[u * n + v]` over `u ∈ S`,
//! which is one matrix–vector product of the adjacency matrix restricted to `S`.
//!
//! The alternating sum is kept non-negative by accumulating the two parities
//! into two separate buckets and subtracting once at the end.

/// `x + y`.
fn add_usize(x: usize, y: usize) -> usize {
    x + y
}

/// `x - y`.
fn sub_usize(x: usize, y: usize) -> usize {
    x - y
}

/// `x * y`.
fn mul_usize(x: usize, y: usize) -> usize {
    x * y
}

/// `x mod m` (the caller passes `m >= 1`).
fn rem_usize(x: usize, m: usize) -> usize {
    let q = x / m;
    x - q * m
}

/// Number of set bits of `x`.
fn popcount_usize(x: usize) -> usize {
    let mut c = x;
    let mut count = 0;
    while c > 0 {
        let bit = c - (c / 2) * 2;
        count += bit;
        c = c / 2;
    }
    count
}

/// Number of Hamiltonian paths of the digraph with adjacency matrix `adj`.
///
/// Returns `1` for `n == 0` (the empty ordering is the unique path).
pub fn count_ham_paths(n: usize, adj: &[usize]) -> usize {
    if n == 0 {
        return 1;
    }
    let nstates = 1usize << n;
    // Buckets: `par[0]` gathers the subsets of even `n - |S|` (sign `+1`),
    // `par[1]` the odd ones (sign `−1`).
    let mut par = vec![0usize; 2];
    let mut s = 0;
    while s < nstates {
        // w[v] counts the sequences of length 1 that stay inside S and end at v.
        let mut w = vec![0usize; n];
        let mut v = 0;
        while v < n {
            let bit = 1usize << v;
            let has = s & bit;
            if has != 0 {
                w[v] = 1;
            }
            v += 1;
        }
        // n - 1 further steps, each one an adjacency-matrix–vector product
        // restricted to S.
        let mut step = 1;
        while step < n {
            let mut nw = vec![0usize; n];
            let mut x = 0;
            while x < n {
                let bx = 1usize << x;
                let hx = s & bx;
                if hx != 0 {
                    let wx = w[x];
                    if wx != 0 {
                        let mut y = 0;
                        while y < n {
                            let by = 1usize << y;
                            let hy = s & by;
                            if hy != 0 {
                                let idx = add_usize(mul_usize(x, n), y);
                                let e = adj[idx];
                                let p = mul_usize(wx, e);
                                let old = nw[y];
                                let t = add_usize(old, p);
                                nw[y] = t;
                            }
                            y += 1;
                        }
                    }
                }
                x += 1;
            }
            w = nw;
            step += 1;
        }
        // W(S) = Σ_v w[v]
        let mut a = 0usize;
        let mut v2 = 0;
        while v2 < n {
            let t = w[v2];
            a = add_usize(a, t);
            v2 += 1;
        }
        let pc = popcount_usize(s);
        let d = sub_usize(n, pc);
        let r = rem_usize(d, 2);
        let old = par[r];
        let np = add_usize(old, a);
        par[r] = np;
        s += 1;
    }
    let res = sub_usize(par[0], par[1]);
    res
}
