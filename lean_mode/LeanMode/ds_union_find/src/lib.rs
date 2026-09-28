//! Disjoint-set union (union-find) with path compression and union by size.
//!
//! Each element starts as its own set.  `find` returns the representative
//! (root) of an element's set, compressing paths along the way; `union`
//! merges two sets, attaching the smaller tree under the larger root.

/// A disjoint-set structure over `0 .. n`.
pub struct DSU {
    pub parent: Vec<usize>,
    pub size: Vec<usize>,
}

/// Create a DSU with `n` singleton sets.
pub fn dsu_new(n: usize) -> DSU {
    let mut parent: Vec<usize> = vec![0; n];
    let mut size: Vec<usize> = vec![1; n];
    let mut i: usize = 0;
    while i < n {
        parent[i] = i;
        i += 1;
    }
    DSU { parent, size }
}

/// Find the representative of `x`, compressing the path from `x` to its root.
///
/// Returns the updated DSU together with the root.
pub fn find(d: DSU, x: usize) -> (DSU, usize) {
    let mut r = d;
    let mut root = x;
    while r.parent[root] != root {
        root = r.parent[root];
    }
    let mut y = x;
    while r.parent[y] != y {
        let next = r.parent[y];
        r.parent[y] = root;
        y = next;
    }
    (r, root)
}

/// Merge the sets of `a` and `b` (union by size).
///
/// Returns the updated DSU and `true` if the two sets were distinct.
pub fn union(d: DSU, a: usize, b: usize) -> (DSU, bool) {
    let (d1, ra) = find(d, a);
    let (mut d2, rb) = find(d1, b);
    if ra == rb {
        (d2, false)
    } else {
        if d2.size[ra] < d2.size[rb] {
            d2.parent[ra] = rb;
            let ns = d2.size[rb] + d2.size[ra];
            d2.size[rb] = ns;
        } else {
            d2.parent[rb] = ra;
            let ns = d2.size[ra] + d2.size[rb];
            d2.size[ra] = ns;
        }
        (d2, true)
    }
}

/// Whether `a` and `b` are in the same set (read-only; no compression).
pub fn connected(d: &DSU, a: usize, b: usize) -> bool {
    let mut ra = a;
    while d.parent[ra] != ra {
        ra = d.parent[ra];
    }
    let mut rb = b;
    while d.parent[rb] != rb {
        rb = d.parent[rb];
    }
    ra == rb
}
