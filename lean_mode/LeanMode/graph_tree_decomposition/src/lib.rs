//! Tree decomposition of an undirected graph, built from an elimination
//! ordering.
//!
//! A *tree decomposition* of a graph `G = (V, E)` is a pair `(T, X)` where `T`
//! is a tree and `X = {X_t}_{t ∈ T}` assigns a non-empty *bag* `X_t ⊆ V` to
//! every node of `T`, such that
//!
//! * (cover) for every edge `{u, v} ∈ E` there is a node `t` with `u, v ∈ X_t`;
//! * (running intersection) for every vertex `v`, the nodes `t` with `v ∈ X_t`
//!   form a connected subtree of `T`.
//!
//! The *width* of `(T, X)` is `max_t |X_t| - 1`; the *treewidth* of `G` is the
//! minimum width over all tree decompositions of `G`.
//!
//! # The elimination-ordering construction
//!
//! Fix an ordering `π = (π₀, π₁, …, π_{n-1})` of `V` and run the *elimination
//! game*: eliminate `π₀` by making its remaining (= later) neighbours a clique
//! (adding the *fill edges*), then `π₁` likewise, and so on.  Let `G_i` be the
//! graph after eliminating `π₀, …, π_{i-1}` and put
//!
//! ```text
//!     bag(i) = { π_i } ∪ { π_j : j > i, π_j is a neighbour of π_i in G_i } .
//! ```
//!
//! Then `bag(0), …, bag(n-1)` together with the parent map
//!
//! ```text
//!     parent(i) = min { j > i : π_j ∈ bag(i) }      (sentinel `n` if none)
//! ```
//!
//! is a tree decomposition of `G` of width `max_i |bag(i)| - 1`; it is *the*
//! tree decomposition of width `tw(G)` when `π` is an optimal elimination
//! ordering.  Note that the bags are built from the *filled* graph: a naive
//! "later neighbours in `G`" bag is **not** a tree decomposition in general
//! (take the path `A–B–C–D` in the order `(A, B, C, D)` — `bag(A) = {A, B, D}`
//! and `bag(C) = {C, D}` both contain `D`, but `bag(B) = {B, C}` does not, so
//! the bags containing `D` are disconnected in the bag path).  The clique
//! completion performed below is what makes the running-intersection property
//! true.
//!
//! The graph is read off a flat, row-major `n × n` symmetric `0/1` adjacency
//! matrix: `adj[u * n + v] == 1` iff `{u, v}` is an edge.  The ordering `π` is
//! a flat slice of `n` distinct vertex ids.  The output is *position-indexed*:
//! bag `i` corresponds to `π_i`, its contents are vertex ids, and both
//! `bag_len` and `parent` have one entry per position.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// A tree decomposition of an `n`-vertex graph with exactly `n` bags.
///
/// * `bag_data` — the concatenated bag contents (vertex ids), bag `0` first;
/// * `bag_len[i]` — the number of vertices in bag `i`;
/// * `parent[i]` — the parent of bag `i` in the bag forest, sentinel `n`
///   (= `bag_len.len()`) when `i` is a root.
pub struct TreeDecomposition {
    pub bag_data: Vec<usize>,
    pub bag_len: Vec<usize>,
    pub parent: Vec<usize>,
}

/// Tree decomposition of the graph given by the flat row-major `n × n`
/// adjacency matrix `adj`, along the elimination ordering `pi`.
///
/// Preconditions (hypotheses of the specification, not checked at runtime):
/// `adj` has length `n * n`, encodes a symmetric irreflexive `0/1` matrix, and
/// `pi` lists each of the `n` vertices exactly once.
pub fn tree_decomposition(n: usize, adj: &[usize], pi: &[usize]) -> TreeDecomposition {
    // `g[i * n + j]` is the entry `(π_i, π_j)` of the current (partially
    // filled) graph, indexed *by position* rather than by vertex id.  Fill
    // edges are only ever created, never deleted, and only inside the strictly
    // later part of the order, so entries below/at the diagonal stay `0`.
    let mut g: Vec<usize> = vec![0usize; n * n];

    let mut i: usize = 0;
    while i < n {
        let u = pi[i];
        let mut j: usize = 0;
        while j < n {
            let w = pi[j];
            let a = adj[idx(u, n, w)];
            let p = idx(i, n, j);
            g[p] = a;
            j += 1;
        }
        i += 1;
    }

    let mut bag_data: Vec<usize> = Vec::new();
    let mut bag_len: Vec<usize> = vec![0usize; n];
    let mut parent: Vec<usize> = vec![n; n];

    let mut s: usize = 0;
    while s < n {
        // Bag `s` = { π_s } ∪ { π_j : j > s, g[s][j] ≠ 0 }.
        let vs = pi[s];
        bag_data.push(vs);
        let mut cnt: usize = 1;
        let mut first: usize = n;
        let mut j: usize = s + 1;
        while j < n {
            let p = idx(s, n, j);
            let e = g[p];
            if e != 0 {
                let wj = pi[j];
                bag_data.push(wj);
                cnt += 1;
                if j < first {
                    first = j;
                }
            }
            j += 1;
        }
        bag_len[s] = cnt;
        parent[s] = first;

        // Eliminate π_s: make its later neighbours a clique (fill edges).
        let mut a: usize = s + 1;
        while a < n {
            let pa = idx(s, n, a);
            let ea = g[pa];
            if ea != 0 {
                let mut b: usize = a + 1;
                while b < n {
                    let pb = idx(s, n, b);
                    let eb = g[pb];
                    if eb != 0 {
                        let ab = idx(a, n, b);
                        g[ab] = 1;
                        let ba = idx(b, n, a);
                        g[ba] = 1;
                    }
                    b += 1;
                }
            }
            a += 1;
        }

        s += 1;
    }

    TreeDecomposition {
        bag_data,
        bag_len,
        parent,
    }
}

// ---------------------------------------------------------------------------
// Optional: the treewidth decision.  `min_elimination_width` brute-forces every
// elimination ordering (the width of an ordering is the width of the tree
// decomposition it generates) and returns the minimum, which is the treewidth
// of the graph.  This is the search side of the classical characterisation
//
//     tw(G) ≤ k  ⟺  some elimination ordering has fill-in width ≤ k.
//
// The enumeration is the recursive backtracking pattern of `array_permutations`
// (recursion on a `&[usize]`-free state struct translates; `next_permutation`
// with its two-pointer reversal was judged too fragile for the supported
// subset).
// ---------------------------------------------------------------------------

/// `true` iff `a < b`; the comparison is routed through a helper so that no
/// comparison ever has a struct-field access as an operand.
fn less_usize(a: usize, b: usize) -> bool {
    if a < b {
        true
    } else {
        false
    }
}

/// State of the exhaustive search over elimination orderings.
struct WidthSearch {
    /// Which values are already placed in the prefix of the ordering.
    used: Vec<bool>,
    /// The ordering under construction.
    current: Vec<usize>,
    /// The smallest width seen so far (sentinel `n` = no ordering yet).
    best: usize,
}

/// Fill position `k` of the ordering with every not-yet-used vertex in turn;
/// at `k == n` the ordering is complete and the width of the tree
/// decomposition it generates is folded into `st.best`.
fn width_search(mut st: WidthSearch, n: usize, adj: &[usize], k: usize) -> WidthSearch {
    if k == n {
        // Copy the ordering out of the state so that it can be passed as a
        // plain slice to `tree_decomposition`.
        let mut order: Vec<usize> = vec![0usize; n];
        let mut i: usize = 0;
        while i < n {
            let x = st.current[i];
            order[i] = x;
            i += 1;
        }
        let td = tree_decomposition(n, adj, &order);
        let mut w: usize = 0;
        let mut j: usize = 0;
        while j < n {
            let l = td.bag_len[j];
            let lm = l - 1;
            if lm > w {
                w = lm;
            }
            j += 1;
        }
        if less_usize(w, st.best) {
            st.best = w;
        }
        st
    } else {
        let mut v: usize = 0;
        while v < n {
            if !st.used[v] {
                st.used[v] = true;
                st.current[k] = v;
                st = width_search(st, n, adj, k + 1);
                st.used[v] = false;
            }
            v += 1;
        }
        st
    }
}

/// The minimum, over all `n!` elimination orderings, of the width of the tree
/// decomposition generated by that ordering — i.e. the treewidth of the graph
/// (brute force; only usable for small `n`, and only meaningful when `n` is
/// smaller than about `9`, where `n!` still fits comfortably).
pub fn min_elimination_width(n: usize, adj: &[usize]) -> usize {
    let used: Vec<bool> = vec![false; n];
    let current: Vec<usize> = vec![0usize; n];
    let st = WidthSearch {
        used,
        current,
        best: n,
    };
    let final_state = width_search(st, n, adj, 0);
    final_state.best
}

// ---------------------------------------------------------------------------
// Untranslated unit tests: the brute-force invariant checker for the tree
// decomposition definition.  These are randomized differential tests, so they
// live here (a `#[cfg(test)] mod` inside `src/lib.rs` is never picked up by the
// translator) and not in `tests/test.rs`, which is translated and `#eval`'d.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Build a flat, row-major, symmetric `0/1` adjacency matrix.
    fn mk_adj(n: usize, edges: &[(usize, usize)]) -> Vec<usize> {
        let mut adj = vec![0usize; n * n];
        for &(a, b) in edges {
            adj[a * n + b] = 1;
            adj[b * n + a] = 1;
        }
        adj
    }

    /// Is `v` a member of bag `i`?
    fn in_bag(td: &TreeDecomposition, i: usize, v: usize) -> bool {
        let mut start = 0usize;
        let mut k = 0usize;
        while k < i {
            start += td.bag_len[k];
            k += 1;
        }
        let len = td.bag_len[i];
        let mut t = 0usize;
        while t < len {
            if td.bag_data[start + t] == v {
                return true;
            }
            t += 1;
        }
        false
    }

    /// Every structural property the specification asserts.
    fn invariants_hold(n: usize, adj: &[usize], pi: &[usize], td: &TreeDecomposition) -> bool {
        // `pi` is a permutation of 0..n.
        let mut seen = vec![0usize; n];
        for &v in pi {
            if v >= n || seen[v] != 0 {
                return false;
            }
            seen[v] = 1;
        }

        // CSR well-formedness.
        let mut total = 0usize;
        for i in 0..n {
            total += td.bag_len[i];
        }
        if total != td.bag_data.len() {
            return false;
        }
        // Every bag is non-empty, inside the vertex range, duplicate-free.
        for i in 0..n {
            if td.bag_len[i] == 0 {
                return false;
            }
            let mut start = 0usize;
            for k in 0..i {
                start += td.bag_len[k];
            }
            for t in 0..td.bag_len[i] {
                let v = td.bag_data[start + t];
                if v >= n {
                    return false;
                }
                for u in (t + 1)..td.bag_len[i] {
                    if td.bag_data[start + u] == v {
                        return false;
                    }
                }
            }
        }

        // The bag forest: `parent i` is a strictly larger index or the sentinel
        // `n`; every node reaches the sentinel.
        for i in 0..n {
            let p = td.parent[i];
            if p > n {
                return false;
            }
            if p < n && p <= i {
                return false;
            }
            let mut cur = i;
            let mut steps = 0usize;
            while cur < n {
                cur = td.parent[cur];
                steps += 1;
                if steps > n {
                    return false;
                }
            }
            if cur != n {
                return false;
            }
        }

        // (cover) every edge lies in some bag.
        for u in 0..n {
            for v in (u + 1)..n {
                if adj[u * n + v] != 0 {
                    let mut found = false;
                    for i in 0..n {
                        if in_bag(td, i, u) && in_bag(td, i, v) {
                            found = true;
                        }
                    }
                    if !found {
                        return false;
                    }
                }
            }
        }

        // (running intersection) the bags containing `v` are connected: they
        // have a top element `t` such that every member's parent is a member
        // and every member reaches `t` walking up.
        for v in 0..n {
            let mut members: Vec<usize> = Vec::new();
            for i in 0..n {
                if in_bag(td, i, v) {
                    members.push(i);
                }
            }
            if members.is_empty() {
                continue;
            }
            let mut top = n;
            for &cand in &members {
                let mut ok = true;
                for &i in &members {
                    // walk up from `i` and see whether `cand` is on the path
                    let mut cur = i;
                    let mut hit = cur == cand;
                    let mut steps = 0usize;
                    while cur < n {
                        cur = td.parent[cur];
                        steps += 1;
                        if cur == cand {
                            hit = true;
                        }
                        if steps > n {
                            break;
                        }
                    }
                    if !hit {
                        ok = false;
                    }
                }
                if ok {
                    top = cand;
                }
            }
            if top == n {
                return false;
            }
            for &i in &members {
                if i != top {
                    let p = td.parent[i];
                    if p >= n || !in_bag(td, p, v) {
                        return false;
                    }
                }
            }
        }

        // (elimination tree) `parent i` is the *smallest* later index whose bag
        // head lies in bag `i`
        for i in 0..n {
            let p = td.parent[i];
            for j in (i + 1)..n {
                if in_bag(td, i, pi[j]) {
                    if p > j {
                        return false;
                    }
                }
            }
            if p < n {
                if !in_bag(td, i, pi[p]) {
                    return false;
                }
            }
        }

        true
    }

    /// xorshift64 — a tiny deterministic PRNG.
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
    }

    #[test]
    fn randomized_invariants() {
        let mut rng = Rng(0x9E3779B97F4A7C15);
        let mut trial = 0usize;
        while trial < 4000 {
            let n = 1 + (rng.next() % 7) as usize; // n in 1..=7
            let mut adj = vec![0usize; n * n];
            for u in 0..n {
                for v in (u + 1)..n {
                    if rng.next() % 2 == 0 {
                        adj[u * n + v] = 1;
                        adj[v * n + u] = 1;
                    }
                }
            }
            // random permutation (Fisher-Yates)
            let mut pi: Vec<usize> = (0..n).collect();
            let mut k = n;
            while k > 1 {
                let j = (rng.next() % (k as u64)) as usize;
                pi.swap(j, k - 1);
                k -= 1;
            }
            let td = tree_decomposition(n, &adj, &pi);
            assert!(
                invariants_hold(n, &adj, &pi, &td),
                "invariants failed: n={} adj={:?} pi={:?} bags={:?}/{:?} par={:?}",
                n,
                adj,
                pi,
                td.bag_data,
                td.bag_len,
                td.parent
            );
            trial += 1;
        }
    }

    /// The naive "later neighbours in `G`" bag construction, with no fill: bag
    /// `i = {π_i} ∪ {π_j : j > i, adj(π_i, π_j) = 1}`, parent = smallest such
    /// `j`.  This is *not* a tree decomposition, and the checker must say so —
    /// otherwise `invariants_hold` would be vacuous.  (It is also why
    /// `tree_decomposition` performs the clique completion.)
    fn naive_unfilled(n: usize, adj: &[usize], pi: &[usize]) -> TreeDecomposition {
        let mut bag_data = Vec::new();
        let mut bag_len = vec![0usize; n];
        let mut parent = vec![n; n];
        for i in 0..n {
            bag_data.push(pi[i]);
            let mut len = 1usize;
            let mut first = n;
            for j in (i + 1)..n {
                if adj[pi[i] * n + pi[j]] != 0 {
                    bag_data.push(pi[j]);
                    len += 1;
                    if j < first {
                        first = j;
                    }
                }
            }
            bag_len[i] = len;
            parent[i] = first;
        }
        TreeDecomposition {
            bag_data,
            bag_len,
            parent,
        }
    }

    #[test]
    fn checker_rejects_unfilled_construction() {
        // The 4-cycle 0-1-2-3-0 in the order (0, 1, 2, 3) = "A-B-C-D": the
        // naive bags are {0,1,3}, {1,2}, {2,3}, {3}, whose parent array is the
        // path 0→1→2→3; vertex 3 sits in bags 0, 2 and 3, but the path from bag
        // 0 to bag 2 runs through bag 1, which lacks 3.
        let n = 4;
        let adj = mk_adj(n, &[(0, 1), (1, 2), (2, 3), (3, 0)]);
        let pi = vec![0usize, 1, 2, 3];
        let naive = naive_unfilled(n, &adj, &pi);
        assert!(!invariants_hold(n, &adj, &pi, &naive));
        // The filled construction *is* a tree decomposition of the same graph.
        let td = tree_decomposition(n, &adj, &pi);
        assert!(invariants_hold(n, &adj, &pi, &td));
        // ... and here the fill is real: bag 0 = {0, 1, 3} (not {0, 1}) and
        // bag 1 = {1, 2, 3} (the fill edge 1-3 lets 3 live in bag 1 too).
        assert_eq!(td.bag_len[0], 3);
        assert_eq!(td.bag_data[0], 0);
        assert_eq!(td.bag_data[1], 1);
        assert_eq!(td.bag_data[2], 3);
        assert_eq!(td.bag_len[1], 3);
        assert_eq!(td.bag_data[3], 1);
        assert_eq!(td.bag_data[4], 2);
        assert_eq!(td.bag_data[5], 3);
        assert_eq!(td.parent[0], 1);
        assert_eq!(td.parent[3], 4);
    }

    /// The width of the produced decomposition is the elimination width of `pi`
    /// (max over positions of `|bag i| - 1`), and it never exceeds `n - 1`.
    #[test]
    fn width_is_bounded() {
        let mut rng = Rng(0xDEADBEEFCAFEF00D);
        let mut trial = 0usize;
        while trial < 500 {
            let n = 1 + (rng.next() % 8) as usize;
            let mut edges: Vec<(usize, usize)> = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    if rng.next() % 3 == 0 {
                        edges.push((u, v));
                    }
                }
            }
            let adj = mk_adj(n, &edges);
            let mut pi: Vec<usize> = (0..n).collect();
            let mut k = n;
            while k > 1 {
                let j = (rng.next() % (k as u64)) as usize;
                pi.swap(j, k - 1);
                k -= 1;
            }
            let td = tree_decomposition(n, &adj, &pi);
            for i in 0..n {
                assert!(td.bag_len[i] >= 1 && td.bag_len[i] <= n);
            }
            trial += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Independent reference for treewidth, used to validate
// `min_elimination_width` on *every* graph with n ≤ 5 vertices.  The reference
// uses the partial-k-tree characterisation
//
//     tw(G) = min { ω(H) - 1 : H ⊇ G, H chordal },
//
// which is a completely different route from "minimise over elimination
// orderings", so agreement is real evidence for the characterisation theorem
// stated in Spec.lean.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tw_reference {
    use super::*;

    /// Is the vertex set `mask` a clique of `adj`?
    fn mask_is_clique(n: usize, adj: &[usize], mask: u32) -> bool {
        for u in 0..n {
            for v in (u + 1)..n {
                let bu = mask & (1u32 << u) != 0;
                let bv = mask & (1u32 << v) != 0;
                if bu && bv && adj[u * n + v] == 0 {
                    return false;
                }
            }
        }
        true
    }

    /// The largest clique size of `adj`.
    fn clique_max(n: usize, adj: &[usize]) -> usize {
        let mut best = 1usize;
        for mask in 0u32..(1u32 << n) {
            let sz = mask.count_ones() as usize;
            if sz > best && mask_is_clique(n, adj, mask) {
                best = sz;
            }
        }
        best
    }

    /// `adj` is chordal iff it has no induced cycle of length ≥ 4: for every
    /// vertex subset of size ≥ 4 whose induced subgraph is 2-regular and
    /// connected (i.e. exactly a chordless cycle), reject.
    fn chordal_ref(n: usize, adj: &[usize]) -> bool {
        for mask in 0u32..(1u32 << n) {
            let sz = mask.count_ones() as usize;
            if sz < 4 {
                continue;
            }
            // induced degrees
            let mut deg2 = true;
            for u in 0..n {
                if mask & (1u32 << u) == 0 {
                    continue;
                }
                let mut d = 0usize;
                for v in 0..n {
                    if v != u && mask & (1u32 << v) != 0 && adj[u * n + v] != 0 {
                        d += 1;
                    }
                }
                if d != 2 {
                    deg2 = false;
                }
            }
            if !deg2 {
                continue;
            }
            // connectivity inside the mask
            let mut seen = 0u32;
            let start = mask.trailing_zeros() as usize;
            let mut stack = vec![start];
            seen |= 1u32 << start;
            while let Some(u) = stack.pop() {
                for v in 0..n {
                    if mask & (1u32 << v) != 0
                        && seen & (1u32 << v) == 0
                        && adj[u * n + v] != 0
                    {
                        seen |= 1u32 << v;
                        stack.push(v);
                    }
                }
            }
            if seen == mask {
                return false; // an induced cycle of length ≥ 4
            }
        }
        true
    }

    /// `tw` by minimising `ω - 1` over chordal supergraphs (partial k-trees).
    fn ref_tw(n: usize, adj: &[usize]) -> usize {
        if n <= 1 {
            return 0;
        }
        // the non-edges of `adj`
        let mut missing: Vec<(usize, usize)> = Vec::new();
        for u in 0..n {
            for v in (u + 1)..n {
                if adj[u * n + v] == 0 {
                    missing.push((u, v));
                }
            }
        }
        let m = missing.len();
        let mut best = n;
        for sub in 0u32..(1u32 << m) {
            let mut h = adj.to_vec();
            for (bit, &(u, v)) in missing.iter().enumerate() {
                if sub & (1u32 << bit) != 0 {
                    h[u * n + v] = 1;
                    h[v * n + u] = 1;
                }
            }
            if chordal_ref(n, &h) {
                let w = clique_max(n, &h) - 1;
                if w < best {
                    best = w;
                }
            }
        }
        best
    }

    /// All graphs on `n` vertices, as flat adjacency matrices, enumerated by
    /// bit pattern of the `C(n,2)` possible edges.
    fn all_graphs(n: usize) -> Vec<Vec<usize>> {
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for u in 0..n {
            for v in (u + 1)..n {
                pairs.push((u, v));
            }
        }
        let m = pairs.len();
        let mut out = Vec::new();
        for bits in 0u32..(1u32 << m) {
            let mut adj = vec![0usize; n * n];
            for (b, &(u, v)) in pairs.iter().enumerate() {
                if bits & (1u32 << b) != 0 {
                    adj[u * n + v] = 1;
                    adj[v * n + u] = 1;
                }
            }
            out.push(adj);
        }
        out
    }

    /// `min_elimination_width` agrees with the chordal-supergraph reference on
    /// *every* graph with at most 5 vertices.
    #[test]
    fn min_elimination_width_matches_reference_exhaustively() {
        for n in 1usize..=5 {
            for adj in all_graphs(n) {
                let got = min_elimination_width(n, &adj);
                let want = ref_tw(n, &adj);
                assert_eq!(got, want, "n={} adj={:?}", n, adj);
            }
        }
    }

    /// Known treewidths.
    #[test]
    fn known_treewidths() {
        // trees and forests
        assert_eq!(min_elimination_width(1, &[0]), 0);
        assert_eq!(min_elimination_width(2, &[0, 1, 1, 0]), 1);
        let path5 = mk_path(5);
        assert_eq!(min_elimination_width(5, &path5), 1);
        // complete graphs
        assert_eq!(min_elimination_width(4, &complete(4)), 3);
        assert_eq!(min_elimination_width(5, &complete(5)), 4);
        // cycles: tw(C_k) = 2 for k ≥ 3
        for k in 3usize..=7 {
            assert_eq!(min_elimination_width(k, &cycle(k)), 2, "cycle {}", k);
        }
        // complete bipartite K_{2,3}
        let mut k23 = vec![0usize; 25];
        for a in 0..2usize {
            for b in 2..5usize {
                k23[a * 5 + b] = 1;
                k23[b * 5 + a] = 1;
            }
        }
        assert_eq!(min_elimination_width(5, &k23), 2);
        // 2 x 3 grid
        let grid23 = grid(2, 3);
        assert_eq!(min_elimination_width(6, &grid23), 2);
        // a 2 x 2 grid is C4
        assert_eq!(min_elimination_width(4, &grid(2, 2)), 2);
        // disconnected: tw = max over components (K4 ⊔ edge)
        let mut d6 = vec![0usize; 36];
        for u in 0..4usize {
            for v in (u + 1)..4usize {
                d6[u * 6 + v] = 1;
                d6[v * 6 + u] = 1;
            }
        }
        d6[4 * 6 + 5] = 1;
        d6[5 * 6 + 4] = 1;
        assert_eq!(min_elimination_width(6, &d6), 3);
        // K_5 minus one edge
        let mut k5m = complete(5);
        k5m[0 * 5 + 1] = 0;
        k5m[1 * 5 + 0] = 0;
        assert_eq!(min_elimination_width(5, &k5m), 3);
        // Petersen graph: tw = 4
        assert_eq!(min_elimination_width(10, &petersen()), 4);
    }

    fn mk_path(n: usize) -> Vec<usize> {
        let mut adj = vec![0usize; n * n];
        for u in 0..n - 1 {
            adj[u * n + u + 1] = 1;
            adj[(u + 1) * n + u] = 1;
        }
        adj
    }

    fn complete(n: usize) -> Vec<usize> {
        let mut adj = vec![0usize; n * n];
        for u in 0..n {
            for v in (u + 1)..n {
                adj[u * n + v] = 1;
                adj[v * n + u] = 1;
            }
        }
        adj
    }

    fn cycle(n: usize) -> Vec<usize> {
        let mut adj = vec![0usize; n * n];
        for u in 0..n {
            let v = (u + 1) % n;
            adj[u * n + v] = 1;
            adj[v * n + u] = 1;
        }
        adj
    }

    fn grid(rows: usize, cols: usize) -> Vec<usize> {
        let n = rows * cols;
        let mut adj = vec![0usize; n * n];
        for r in 0..rows {
            for c in 0..cols {
                let u = r * cols + c;
                if c + 1 < cols {
                    let v = r * cols + c + 1;
                    adj[u * n + v] = 1;
                    adj[v * n + u] = 1;
                }
                if r + 1 < rows {
                    let v = (r + 1) * cols + c;
                    adj[u * n + v] = 1;
                    adj[v * n + u] = 1;
                }
            }
        }
        adj
    }

    fn petersen() -> Vec<usize> {
        let n = 10;
        let mut adj = vec![0usize; n * n];
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 4), (4, 0), // outer 5-cycle
            (5, 7), (7, 9), (9, 6), (6, 8), (8, 5), // inner pentagram
            (0, 5), (1, 6), (2, 7), (3, 8), (4, 9), // spokes
        ];
        for &(a, b) in edges.iter() {
            adj[a * n + b] = 1;
            adj[b * n + a] = 1;
        }
        adj
    }
}
