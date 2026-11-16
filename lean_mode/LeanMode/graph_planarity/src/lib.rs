//! Planarity testing by complete minor testing for `K5` and `K3,3`.
//!
//! The graph on `n` vertices is given as a flat, row-major, symmetric `n × n`
//! 0/1 adjacency matrix `adj`: `adj[u * n + v] == 1` iff the undirected edge
//! `{u, v}` is present (the diagonal is 0, i.e. no self-loops).
//!
//! By Wagner's form of the Kuratowski theorem a finite simple graph is planar
//! **iff** it has neither `K5` nor `K3,3` as a minor.  A graph `G` has `H` as a
//! minor iff its vertices can be split into `|H|` non-empty *parts*, each part
//! inducing a connected subgraph of `G`, such that every edge of `H` is
//! realised by an edge of `G` between the corresponding parts (the remaining
//! vertices of `G` are "unused").
//!
//! ## The search performed here
//!
//! For the two forbidden graphs the *existence* of such a model can be decided
//! on set partitions of `V(G)` alone:
//!
//! * `K5` — the five vertices of `K5` are mutually adjacent and may be matched
//!   to any five classes, so a model exists iff some five classes are connected
//!   and pairwise joined;
//! * `K3,3` — six classes are needed, and the only remaining choice is which
//!   three of them form the first part of the bipartition, so a model exists
//!   iff six classes are connected split `3 + 3` with every cross pair joined.
//!
//! The unused vertices may always be collected into a single extra class, so
//! it is enough to look at partitions into at most `|H| + 1` classes.  Those
//! are enumerated **canonically** (restricted growth strings: the labels are
//! introduced in order of first use), which visits only `Bell(n)` partitions —
//! 203 for `n = 6`, instead of the `(|H|+1)^n` labelings of a naive model
//! enumeration.  The enumeration is complete: the canonical partition of any
//! solution is one of the visited ones.

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// `x + 1`, routed through a function so that the increment is a `usize`
/// operation on a parameter.
fn add1(x: usize) -> usize {
    let y = x + 1;
    y
}

/// The largest label allowed at position `idx` inside a restricted growth
/// string: position `0` must carry label `0`, and every later position may
/// introduce at most one new label beyond the maximum seen so far, capped at
/// `base - 1` (the largest available label).
fn label_cap(assign: &[usize], idx: usize, base: usize) -> usize {
    if idx == 0 {
        0
    } else {
        let mut mx: usize = 0;
        let mut t: usize = 0;
        while t < idx {
            let v = assign[t];
            if v > mx {
                mx = v;
            }
            t += 1;
        }
        let lim = base - 1;
        if mx < lim {
            add1(mx)
        } else {
            lim
        }
    }
}

/// Advance the restricted growth string `assign` to the next partition in
/// lexicographic order and report whether there was one.  The successor is
/// found by increasing the rightmost position that can still grow, after
/// zeroing every position to its right (carry propagation).
fn next_partition(mut assign: Vec<usize>, n: usize, base: usize) -> (Vec<usize>, bool) {
    let mut pos: usize = n;
    let mut j: usize = n;
    while j > 0 {
        if pos == n {
            let i = j - 1;
            let cap = label_cap(&assign, i, base);
            let cur = assign[i];
            if cur < cap {
                let nv = add1(cur);
                assign[i] = nv;
                pos = i;
            } else {
                assign[i] = 0;
            }
        }
        j -= 1;
    }
    let mut ok: bool = false;
    if pos < n {
        ok = true;
    }
    (assign, ok)
}

/// The number of classes of a canonical partition: its labels are exactly
/// `0 .. class_count - 1`.
fn class_count(assign: &[usize], n: usize) -> usize {
    let mut mx: usize = 0;
    let mut i: usize = 0;
    while i < n {
        let v = assign[i];
        if v > mx {
            mx = v;
        }
        i += 1;
    }
    add1(mx)
}

/// `1` when class `p` is non-empty and the vertices of class `p` induce a
/// connected subgraph of `G`; `0` otherwise.
fn part_connected(n: usize, adj: &[usize], assign: &[usize], p: usize) -> usize {
    // First vertex of the class (`n` is the "empty class" sentinel).
    let mut s0: usize = n;
    let mut i: usize = 0;
    while i < n {
        if s0 == n {
            let a = assign[i];
            if a == p {
                s0 = i;
            }
        }
        i += 1;
    }
    if s0 == n {
        0
    } else {
        let mut seen: Vec<usize> = vec![0usize; n];
        let mut queue: Vec<usize> = vec![0usize; n];
        seen[s0] = 1;
        let mut head: usize = 0;
        let mut tail: usize = 0;
        queue[tail] = s0;
        tail += 1;
        while head < tail {
            let u = queue[head];
            head += 1;
            let mut v: usize = 0;
            while v < n {
                let s = seen[v];
                if s == 0 {
                    let a = assign[v];
                    if a == p {
                        let e = adj[idx(u, n, v)];
                        if e == 1 {
                            seen[v] = 1;
                            queue[tail] = v;
                            tail += 1;
                        }
                    }
                }
                v += 1;
            }
        }
        // Every vertex of the class must have been reached.
        let mut ok: usize = 1;
        let mut j: usize = 0;
        while j < n {
            let a = assign[j];
            if a == p {
                let s = seen[j];
                if s == 0 {
                    ok = 0;
                }
            }
            j += 1;
        }
        ok
    }
}

/// `1` when some edge of `G` joins a vertex of class `a` to a vertex of class
/// `b`; `0` otherwise.
fn parts_joined(n: usize, adj: &[usize], assign: &[usize], a: usize, b: usize) -> usize {
    let mut hit: usize = 0;
    let mut u: usize = 0;
    while u < n {
        let au = assign[u];
        if au == a {
            let mut v: usize = 0;
            while v < n {
                let av = assign[v];
                if av == b {
                    let x = adj[idx(u, n, v)];
                    if x == 1 {
                        hit = 1;
                    }
                }
                v += 1;
            }
        }
        u += 1;
    }
    hit
}

/// Is `x` one of the three labels `i`, `j`, `k`?
fn on_side_a(x: usize, i: usize, j: usize, k: usize) -> bool {
    if x == i {
        true
    } else {
        if x == j {
            true
        } else {
            if x == k {
                true
            } else {
                false
            }
        }
    }
}

/// Do the five classes of the partition other than `skip` form a `K5` model
/// (each connected, every two joined)?  `skip == r` means that no class is
/// skipped, i.e. that the partition has exactly five classes.
fn k5_choose(n: usize, adj: &[usize], assign: &[usize], r: usize, skip: usize) -> usize {
    let mut conn: usize = 1;
    let mut p: usize = 0;
    while p < r {
        if p != skip {
            let c = part_connected(n, adj, assign, p);
            if c == 0 {
                conn = 0;
            }
        }
        p += 1;
    }
    let mut res: usize = 0;
    if conn != 0 {
        res = 1;
        let mut a: usize = 0;
        while a < r {
            if a != skip {
                let mut b: usize = 0;
                while b < r {
                    if b > a {
                        if b != skip {
                            let e = parts_joined(n, adj, assign, a, b);
                            if e == 0 {
                                res = 0;
                            }
                        }
                    }
                    b += 1;
                }
            }
            a += 1;
        }
    }
    res
}

/// Are all cross pairs of the `3 + 3` split of classes given by `i`, `j`, `k`
/// (the first side) joined?  The other side is the remaining three classes of
/// the partition, `skip` excluded.
fn k33_split(
    n: usize,
    adj: &[usize],
    assign: &[usize],
    r: usize,
    skip: usize,
    i: usize,
    j: usize,
    k: usize,
) -> usize {
    let mut ok: usize = 1;
    let mut a: usize = 0;
    while a < r {
        if a != skip {
            let a_a = on_side_a(a, i, j, k);
            let mut b: usize = 0;
            while b < r {
                if b != skip {
                    let b_a = on_side_a(b, i, j, k);
                    if b_a != a_a {
                        let e = parts_joined(n, adj, assign, a, b);
                        if e == 0 {
                            ok = 0;
                        }
                    }
                }
                b += 1;
            }
        }
        a += 1;
    }
    ok
}

/// Do the six classes of the partition other than `skip` form a `K3,3` model
/// (each connected, and some `3 + 3` split with every cross pair joined)?
fn k33_choose(n: usize, adj: &[usize], assign: &[usize], r: usize, skip: usize) -> usize {
    let mut conn: usize = 1;
    let mut p: usize = 0;
    while p < r {
        if p != skip {
            let c = part_connected(n, adj, assign, p);
            if c == 0 {
                conn = 0;
            }
        }
        p += 1;
    }
    let mut res: usize = 0;
    if conn != 0 {
        let mut i: usize = 0;
        while i < r {
            if i != skip {
                let mut j: usize = 0;
                while j < r {
                    if j > i {
                        if j != skip {
                            let mut k: usize = 0;
                            while k < r {
                                if k > j {
                                    if k != skip {
                                        let v = k33_split(n, adj, assign, r, skip, i, j, k);
                                        if v != 0 {
                                            res = 1;
                                        }
                                    }
                                }
                                k += 1;
                            }
                        }
                    }
                    j += 1;
                }
            }
            i += 1;
        }
    }
    res
}

/// Does `G` have a `K5` minor?  `G` has one iff some partition of `V(G)` into
/// at most six classes has five classes that are connected and pairwise joined.
fn has_k5_minor(n: usize, adj: &[usize]) -> bool {
    let base: usize = 6;
    let mut assign: Vec<usize> = vec![0usize; n];
    let mut more: bool = true;
    let mut found: bool = false;
    while more {
        let r = class_count(&assign, n);
        let mut v: usize = 0;
        if r == 5 {
            v = k5_choose(n, adj, &assign, r, r);
        } else {
            if r == 6 {
                let mut s: usize = 0;
                while s < r {
                    let x = k5_choose(n, adj, &assign, r, s);
                    if x != 0 {
                        v = 1;
                    }
                    s += 1;
                }
            }
        }
        if v != 0 {
            found = true;
        }
        let (a1, m1) = next_partition(assign, n, base);
        assign = a1;
        more = m1;
    }
    found
}

/// Does `G` have a `K3,3` minor?  `G` has one iff some partition of `V(G)` into
/// at most seven classes has six classes that are connected and admit a
/// `3 + 3` split with every cross pair joined.
fn has_k33_minor(n: usize, adj: &[usize]) -> bool {
    let base: usize = 7;
    let mut assign: Vec<usize> = vec![0usize; n];
    let mut more: bool = true;
    let mut found: bool = false;
    while more {
        let r = class_count(&assign, n);
        let mut v: usize = 0;
        if r == 6 {
            v = k33_choose(n, adj, &assign, r, r);
        } else {
            if r == 7 {
                let mut s: usize = 0;
                while s < r {
                    let x = k33_choose(n, adj, &assign, r, s);
                    if x != 0 {
                        v = 1;
                    }
                    s += 1;
                }
            }
        }
        if v != 0 {
            found = true;
        }
        let (a1, m1) = next_partition(assign, n, base);
        assign = a1;
        more = m1;
    }
    found
}

/// Is the graph planar?
///
/// Returns `true` iff `G` has neither a `K5` minor nor a `K3,3` minor
/// (Wagner's form of the Kuratowski theorem), which for a finite simple graph
/// is the same as being drawable in the plane without crossing edges.
pub fn is_planar(n: usize, adj: &[usize]) -> bool {
    let has_k5 = has_k5_minor(n, adj);
    let has_k33 = has_k33_minor(n, adj);
    let bad = has_k5 || has_k33;
    if bad {
        false
    } else {
        true
    }
}

/// Unit tests: brute-force reference implementations and randomized
/// differential tests.  This module is *not* picked up by the translator (only
/// `tests/test.rs` is), so it may use iterators, closures and `assert!` freely.
#[cfg(test)]
mod tests {
    use super::*;

    /// Reference flat index (independent of `idx`).
    fn ref_at(adj: &[usize], n: usize, u: usize, v: usize) -> usize {
        adj[u * n + v]
    }

    /// `true` when the edge `{u, v}` (u < v) is present.
    fn ref_edge(adj: &[usize], n: usize, u: usize, v: usize) -> bool {
        ref_at(adj, n, u, v) == 1
    }

    fn ref_find(parent: &mut Vec<usize>, x: usize) -> usize {
        let mut r = x;
        while parent[r] != r {
            r = parent[r];
        }
        let mut c = x;
        while parent[c] != c {
            let nx = parent[c];
            parent[c] = r;
            c = nx;
        }
        r
    }

    fn ref_subgraph_iso(
        image: &[Vec<bool>],
        h_edges: &[(usize, usize)],
        k: usize,
        idx: usize,
        img: &mut Vec<usize>,
    ) -> bool {
        if idx == k {
            return true;
        }
        for w in 0..image.len() {
            if img[..idx].contains(&w) {
                continue;
            }
            img[idx] = w;
            // Only the edges whose larger endpoint is `idx` can be checked
            // now; the others are checked when that endpoint is assigned.
            let ok = h_edges
                .iter()
                .filter(|&&(_, b)| b == idx)
                .all(|&(a, b)| image[img[a]][img[b]]);
            if ok && ref_subgraph_iso(image, h_edges, k, idx + 1, img) {
                return true;
            }
            img[idx] = usize::MAX;
        }
        false
    }

    /// Contract the edge set `fs` (a bitmask over the `edges` list) and test
    /// whether the resulting multigraph contains `H` as a subgraph.
    ///
    /// A graph has `H` as a minor iff some contraction of `G` (a partition of
    /// `V(G)` whose classes are connected) contains `H` as a subgraph —
    /// deletions being implicit in subgraph containment.  This reference
    /// therefore enumerates *every* subset of edges to contract and checks
    /// subgraph containment: an algorithm completely different from the
    /// partition search under test.
    fn ref_contains_after_contraction(
        n: usize,
        edges: &[(usize, usize)],
        fs: usize,
        h_edges: &[(usize, usize)],
        k: usize,
    ) -> bool {
        let mut parent: Vec<usize> = (0..n).collect();
        for (e, &(a, b)) in edges.iter().enumerate() {
            if (fs >> e) & 1 == 1 {
                let ra = ref_find(&mut parent, a);
                let rb = ref_find(&mut parent, b);
                parent[ra] = rb;
            }
        }
        let mut comp: Vec<usize> = vec![usize::MAX; n];
        let mut nc = 0usize;
        for v in 0..n {
            let r = ref_find(&mut parent, v);
            if comp[r] == usize::MAX {
                comp[r] = nc;
                nc += 1;
            }
        }
        let mut image = vec![vec![false; nc]; nc];
        for &(a, b) in edges {
            let ca = comp[ref_find(&mut parent, a)];
            let cb = comp[ref_find(&mut parent, b)];
            if ca != cb {
                image[ca][cb] = true;
                image[cb][ca] = true;
            }
        }
        ref_subgraph_iso(&image, h_edges, k, 0, &mut vec![usize::MAX; k])
    }

    /// Reference `has K5 or K3,3 minor` by exhaustive contraction.
    fn ref_non_planar(n: usize, adj: &[usize]) -> bool {
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for u in 0..n {
            for v in (u + 1)..n {
                if ref_edge(adj, n, u, v) {
                    edges.push((u, v));
                }
            }
        }
        let k5: Vec<(usize, usize)> = (0..5)
            .flat_map(|a| ((a + 1)..5).map(move |b| (a, b)))
            .collect();
        let k33: Vec<(usize, usize)> = (0..3)
            .flat_map(|a| (3..6).map(move |b| (a, b)))
            .collect();
        let m = edges.len();
        for fs in 0usize..(1usize << m) {
            if ref_contains_after_contraction(n, &edges, fs, &k5, 5) {
                return true;
            }
            if ref_contains_after_contraction(n, &edges, fs, &k33, 6) {
                return true;
            }
        }
        false
    }

    /// Deterministic 64-bit PRNG (splitmix64) — no external crates.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        }

        fn below(&mut self, b: u64) -> u64 {
            self.next() % b
        }
    }

    /// Random symmetric 0/1 matrix with zero diagonal.
    fn random_graph(n: usize, pct: u64, rng: &mut Rng) -> Vec<usize> {
        let mut adj = vec![0usize; n * n];
        for u in 0..n {
            for v in (u + 1)..n {
                if rng.below(100) < pct {
                    adj[u * n + v] = 1;
                    adj[v * n + u] = 1;
                }
            }
        }
        adj
    }

    /// The successor-based enumeration must produce exactly the canonical
    /// labelings (position 0 carries label 0, every later position may
    /// introduce at most one new label) in lexicographic order, without
    /// repetitions and without omissions.
    #[test]
    fn canonical_partition_enumeration_matches_characterisation() {
        for n in 0..=6usize {
            for base in 2..=7usize {
                let total = base.pow(n as u32);
                let mut want: Vec<Vec<usize>> = Vec::new();
                for code in 0..total {
                    let mut a = vec![0usize; n];
                    let mut c = code;
                    for i in 0..n {
                        a[i] = c % base;
                        c /= base;
                    }
                    let mut ok = !(n > 0 && a[0] != 0);
                    for i in 1..n {
                        let mx = a[..i].iter().copied().max().unwrap();
                        if a[i] > mx + 1 {
                            ok = false;
                        }
                    }
                    if ok {
                        want.push(a);
                    }
                }
                // `code` decodes least-significant-label first, so sort the
                // expected sequences into the successor's lexicographic order.
                want.sort();

                let mut got: Vec<Vec<usize>> = Vec::new();
                let mut assign = vec![0usize; n];
                let mut more = true;
                let mut guard = 0usize;
                while more {
                    got.push(assign.clone());
                    let (a1, m1) = next_partition(assign, n, base);
                    assign = a1;
                    more = m1;
                    guard += 1;
                    assert!(guard <= total + 1, "enumeration does not terminate");
                }
                assert_eq!(got, want, "n={} base={}", n, base);
            }
        }
    }

    /// --- A direct transcription of the `Spec.lean` definition of planarity ---
    ///
    /// `Spec.lean` defines `IsPlanar A := ∃ rot, IsRotationSystem A rot ∧
    /// EulerCharacteristicOK A rot`, where the faces are the orbits of the
    /// face permutation `(u, v) ↦ (v, rot v u)` on the darts and `V - E + F =
    /// 2c` must hold (isolated vertices counting one face each).  The code
    /// below decides that definition by brute force over the `n^n` candidate
    /// rotation systems, so the spec can be checked against the program
    /// directly on the classical examples.

    /// Lexicographic order on darts, as in `Spec.lean`'s `dartLE`.
    fn spec_dart_le(x: (usize, usize), y: (usize, usize)) -> bool {
        x.0 < y.0 || (x.0 == y.0 && x.1 <= y.1)
    }

    /// The face permutation `(u, v) ↦ (v, rot v u)`.
    fn spec_face_step(n: usize, rot: &[usize], d: (usize, usize)) -> (usize, usize) {
        (d.1, rot[d.1 * n + d.0])
    }

    fn spec_is_rotation_system(n: usize, adj: &[usize], rot: &[usize]) -> bool {
        for v in 0..n {
            for u in 0..n {
                if ref_at(adj, n, v, u) == 1 {
                    if ref_at(adj, n, v, rot[v * n + u]) != 1 {
                        return false;
                    }
                }
            }
            // The orbit of every neighbour of `v` must cover every neighbour.
            for u in 0..n {
                if ref_at(adj, n, v, u) == 1 {
                    for w in 0..n {
                        if ref_at(adj, n, v, w) == 1 {
                            let mut x = u;
                            let mut reached = false;
                            for _ in 0..=n {
                                if x == w {
                                    reached = true;
                                }
                                x = rot[v * n + x];
                            }
                            if !reached {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    /// Number of faces: orbits of the face permutation, counted at their
    /// lexicographically least dart.
    fn spec_face_count(n: usize, adj: &[usize], rot: &[usize]) -> usize {
        let mut count = 0usize;
        for a in 0..n {
            for b in 0..n {
                if ref_at(adj, n, a, b) == 1 {
                    let mut x = (a, b);
                    let mut is_least = true;
                    for _ in 0..=(n * n) {
                        if !spec_dart_le((a, b), x) {
                            is_least = false;
                        }
                        x = spec_face_step(n, rot, x);
                    }
                    if is_least {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn spec_euler_ok(n: usize, adj: &[usize], rot: &[usize]) -> bool {
        let mut edges = 0usize;
        for u in 0..n {
            for v in (u + 1)..n {
                if ref_at(adj, n, u, v) == 1 {
                    edges += 1;
                }
            }
        }
        let mut isolated = 0usize;
        for v in 0..n {
            let mut deg = 0usize;
            for u in 0..n {
                if ref_at(adj, n, v, u) == 1 {
                    deg += 1;
                }
            }
            if deg == 0 {
                isolated += 1;
            }
        }
        // connected components (undirected reachability, iterative BFS)
        let mut comp = vec![usize::MAX; n];
        let mut ncomp = 0usize;
        for s in 0..n {
            if comp[s] == usize::MAX {
                comp[s] = ncomp;
                let mut stack = vec![s];
                while let Some(x) = stack.pop() {
                    for y in 0..n {
                        if ref_at(adj, n, x, y) == 1 && comp[y] == usize::MAX {
                            comp[y] = ncomp;
                            stack.push(y);
                        }
                    }
                }
                ncomp += 1;
            }
        }
        let faces = spec_face_count(n, adj, rot) + isolated;
        n + faces == edges + 2 * ncomp
    }

    fn fact(k: usize) -> usize {
        let mut r = 1usize;
        for i in 2..=k {
            r *= i;
        }
        r
    }

    /// Decide `Spec.lean`'s `IsPlanar` by brute force.
    ///
    /// `IsRotationSystem` says exactly that `rot v` is a single cycle on the
    /// neighbours of `v` (its two clauses are "neighbours map to neighbours"
    /// and "the orbit of a neighbour covers all neighbours"), and the face
    /// permutation only ever reads `rot` at neighbour slots (the new dart
    /// `(v, rot v u)` is a dart, so `v` is a neighbour of its head).  Hence it
    /// is faithful to enumerate, at every vertex, the `(deg - 1)!` cyclic
    /// orders of the incident edges (written with the smallest neighbour
    /// first) and test the Euler condition on each combination.
    fn spec_is_planar(n: usize, adj: &[usize]) -> bool {
        let mut nb: Vec<Vec<usize>> = Vec::new();
        let mut radix: Vec<usize> = Vec::new();
        for v in 0..n {
            let l: Vec<usize> = (0..n).filter(|&u| ref_at(adj, n, v, u) == 1).collect();
            let d = l.len();
            radix.push(fact(d.saturating_sub(1)));
            nb.push(l);
        }
        let total: usize = radix.iter().product();
        for code in 0..total {
            let mut rot = vec![0usize; n * n];
            let mut c = code;
            for v in 0..n {
                let d = nb[v].len();
                let idx = c % radix[v];
                c /= radix[v];
                if d >= 2 {
                    // factoradic decoding of the permutation of nb[v][1..]
                    let mut k = idx;
                    let mut rest: Vec<usize> = nb[v][1..].to_vec();
                    let mut order: Vec<usize> = vec![nb[v][0]];
                    let mut i = d - 1;
                    while i > 0 {
                        let f = fact(i - 1);
                        let pick = k / f;
                        k %= f;
                        order.push(rest.remove(pick));
                        i -= 1;
                    }
                    for t in 0..d {
                        rot[v * n + order[t]] = order[(t + 1) % d];
                    }
                } else if d == 1 {
                    rot[v * n + nb[v][0]] = nb[v][0];
                }
            }
            // Every candidate must really be a rotation system.
            assert!(
                spec_is_rotation_system(n, adj, &rot),
                "candidate is not a rotation system"
            );
            if spec_euler_ok(n, adj, &rot) {
                return true;
            }
        }
        false
    }

    fn mk(n: usize, edges: &[(usize, usize)]) -> Vec<usize> {
        let mut adj = vec![0usize; n * n];
        for &(u, v) in edges {
            adj[u * n + v] = 1;
            adj[v * n + u] = 1;
        }
        adj
    }

    /// The classical examples, checked against the *spec's* definition of
    /// planarity (not against the algorithm's minor criterion).
    #[test]
    fn spec_definition_matches_program_on_classical_cases() {
        let cases: Vec<(usize, Vec<usize>, bool)> = vec![
            (3, mk(3, &[(0, 1), (1, 2), (2, 0)]), true),                        // K3
            (4, mk(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]), true), // K4
            (5, mk(5, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 0)]), true),         // C5
            (
                5,
                mk(5, &[(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]),
                false,                                                           // K5
            ),
            (
                5,
                mk(5, &[(0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]),
                true,                                                            // K5 - e
            ),
            (6, mk(6, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)]), true), // C6
            (
                6,
                mk(6, &[(0, 3), (0, 4), (0, 5), (1, 3), (1, 4), (1, 5), (2, 3), (2, 4), (2, 5)]),
                false,                                                           // K3,3
            ),
            (
                6,
                mk(6, &[(0, 4), (0, 5), (1, 3), (1, 4), (1, 5), (2, 3), (2, 4), (2, 5)]),
                true,                                                            // K3,3 - e
            ),
            (
                6,
                mk(
                    6,
                    &[
                        (0, 2), (0, 3), (0, 4), (0, 5), (1, 2), (1, 3), (1, 4), (1, 5), (2, 4),
                        (2, 5), (3, 4), (3, 5),
                    ],
                ),
                true,                                                            // octahedron
            ),
            (
                6,
                mk(
                    6,
                    &[
                        (0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4), (2, 3), (2, 4),
                        (3, 4),
                    ],
                ),
                false,                                                           // K5 + isolated
            ),
            (
                8,
                mk(
                    8,
                    &[
                        (0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4),
                        (1, 5), (2, 6), (3, 7),
                    ],
                ),
                true,                                                            // cube Q3
            ),
        ];
        for (n, adj, want) in cases {
            let spec = spec_is_planar(n, &adj);
            let prog = is_planar(n, &adj);
            assert_eq!(spec, want, "spec definition on a {} - vertex graph", n);
            assert_eq!(prog, want, "program on a {} - vertex graph", n);
        }
    }

    #[test]
    fn exhaustive_small_graphs_match_reference() {
        // All graphs on n <= 5 vertices, and random graphs on n = 6, are
        // compared against the contraction-based reference.
        for n in 0..=5usize {
            let pairs: Vec<(usize, usize)> =
                (0..n).flat_map(|u| ((u + 1)..n).map(move |v| (u, v))).collect();
            for mask in 0usize..(1usize << pairs.len()) {
                let mut adj = vec![0usize; n * n];
                for (e, &(u, v)) in pairs.iter().enumerate() {
                    if (mask >> e) & 1 == 1 {
                        adj[u * n + v] = 1;
                        adj[v * n + u] = 1;
                    }
                }
                let got = is_planar(n, &adj);
                let want = !ref_non_planar(n, &adj);
                assert_eq!(got, want, "n={} mask={}", n, mask);
            }
        }
    }

    #[test]
    fn random_six_vertex_graphs_match_reference() {
        // Half the samples are subgraphs of a fixed planar graph (so planar),
        // half are built around K3,3 or K5 (so non-planar): both answers are
        // exercised on six vertices, where K3,3 first fits.
        let mut rng = Rng(0x1234_5678_9ABC_DEF0);
        let mut octahedron = vec![0usize; 36];
        for u in 0..6 {
            for v in (u + 1)..6 {
                let portal = u / 2 == v / 2;
                if !portal {
                    octahedron[u * 6 + v] = 1;
                    octahedron[v * 6 + u] = 1;
                }
            }
        }
        let mut k33 = vec![0usize; 36];
        for u in 0..3 {
            for v in 3..6 {
                k33[u * 6 + v] = 1;
                k33[v * 6 + u] = 1;
            }
        }
        for trial in 0..24 {
            let planarity_sample = trial % 2 == 0;
            let mut adj = if planarity_sample {
                octahedron.clone()
            } else {
                k33.clone()
            };
            for u in 0..6 {
                for v in (u + 1)..6 {
                    if rng.below(100) < 35 {
                        // Planar samples only lose edges, non-planar samples
                        // only gain them.
                        let x = if planarity_sample { 0 } else { 1 };
                        adj[u * 6 + v] = x;
                        adj[v * 6 + u] = x;
                    }
                }
            }
            let got = is_planar(6, &adj);
            let want = !ref_non_planar(6, &adj);
            assert_eq!(got, want, "trial={} planar_sample={}", trial, planarity_sample);
            if planarity_sample {
                assert!(got, "a subgraph of a planar graph is planar");
            }
        }
        // Purely random six-vertex graphs, against the reference.
        for trial in 0..6 {
            let pct = 30 + rng.below(45);
            let adj = random_graph(6, pct, &mut rng);
            let got = is_planar(6, &adj);
            let want = !ref_non_planar(6, &adj);
            assert_eq!(got, want, "random trial={} pct={}", trial, pct);
        }
    }

    #[test]
    fn planarity_known_families() {
        // Planar: every cycle is planar.
        for n in 3..=6usize {
            let mut adj = vec![0usize; n * n];
            for u in 0..n {
                let v = (u + 1) % n;
                adj[u * n + v] = 1;
                adj[v * n + u] = 1;
            }
            assert!(is_planar(n, &adj), "cycle c{} should be planar", n);
        }
        // Non-planar: K5.
        let k5 = [
            [0, 1, 1, 1, 1],
            [1, 0, 1, 1, 1],
            [1, 1, 0, 1, 1],
            [1, 1, 1, 0, 1],
            [1, 1, 1, 1, 0],
        ];
        let mut flat = vec![0usize; 25];
        for u in 0..5 {
            for v in 0..5 {
                flat[u * 5 + v] = k5[u][v];
            }
        }
        assert!(!is_planar(5, &flat));
        // Subdividing the edge {0, 1} of K5 yields a graph with a K5 minor.
        let mut sub = vec![0usize; 36];
        for u in 0..5 {
            for v in 0..5 {
                sub[u * 6 + v] = k5[u][v];
            }
        }
        sub[0 * 6 + 1] = 0;
        sub[1 * 6 + 0] = 0;
        sub[5 * 6 + 0] = 1;
        sub[0 * 6 + 5] = 1;
        sub[5 * 6 + 1] = 1;
        sub[1 * 6 + 5] = 1;
        assert!(!is_planar(6, &sub));
        // K3,3 is non-planar, but deleting one of its edges makes it planar
        // (K3,3 too is edge-minimal non-planar).
        let mut k33 = vec![0usize; 36];
        for u in 0..3 {
            for v in 3..6 {
                k33[u * 6 + v] = 1;
                k33[v * 6 + u] = 1;
            }
        }
        assert!(!is_planar(6, &k33));
        k33[0 * 6 + 3] = 0;
        k33[3 * 6 + 0] = 0;
        assert!(is_planar(6, &k33), "K3,3 minus an edge is planar");
    }

    #[test]
    fn planar_subgraphs_of_k5_are_planar() {
        // Deleting any single edge of K5 gives a planar graph (K5 is
        // edge-minimal non-planar).
        let mut flat = vec![1usize; 25];
        for u in 0..5 {
            flat[u * 5 + u] = 0;
        }
        for a in 0..5 {
            for b in (a + 1)..5 {
                let mut adj = flat.clone();
                adj[a * 5 + b] = 0;
                adj[b * 5 + a] = 0;
                assert!(is_planar(5, &adj), "K5 minus edge ({},{})", a, b);
            }
        }
        // K5 minus an edge, plus an isolated vertex: still planar.
        let mut adj = flat.clone();
        adj[0 * 5 + 1] = 0;
        adj[1 * 5 + 0] = 0;
        let mut big = vec![0usize; 36];
        for u in 0..5 {
            for v in 0..5 {
                big[u * 6 + v] = adj[u * 5 + v];
            }
        }
        assert!(is_planar(6, &big));
    }
}
