//! Binomial heap.
//!
//! A binomial tree of degree (rank) `r` is a root whose children are binomial
//! trees of degrees `r-1, r-2, …, 0`; hence such a tree has exactly `2^r`
//! nodes.  A binomial heap is a forest of these trees in which the roots have
//! *pairwise distinct* degrees — the binary-counter invariant that keeps the
//! forest logarithmic in size.
//!
//! The node pool is a set of flat `Vec`s indexed by node number `1 ..= n`, with
//! `0` as the null sentinel.  Each node stores its key, its `degree` (number of
//! children, also the rank of the binomial tree it roots), its `parent`, its
//! first (highest-degree) `child` and its next `sibling`.  The forest itself is
//! a separate `Vec` of root indices, kept sorted by increasing degree.
//!
//! Operations:
//! * `binom_merge`   — union of two heaps (splice pools, then consolidate);
//! * `binom_insert`  — merge in a fresh single-node tree;
//! * `binom_find_min`/`binom_extract_min` — read / remove one occurrence of the
//!   global minimum (the popped root's children are promoted to roots).

/// A binomial heap: a flat node pool (`1 ..= n`), a root forest, and `n`, the
/// number of pool slots in use.
pub struct BinomHeap {
    /// `key[v]` — the key stored at node `v`.
    pub key: Vec<i64>,
    /// `degree[v]` — number of children of `v` (the rank of its subtree).
    pub degree: Vec<usize>,
    /// `parent[v]` — parent of `v` (`0` for a root).
    pub parent: Vec<usize>,
    /// `child[v]` — first (highest-degree) child of `v` (`0` if `v` is a leaf).
    pub child: Vec<usize>,
    /// `sibling[v]` — next sibling of `v` (`0` for the last one).
    pub sibling: Vec<usize>,
    /// Root indices, sorted by increasing degree.
    pub roots: Vec<usize>,
    /// Number of pool slots in use (nodes are `1 ..= n`).
    pub n: usize,
}

/// `a + b` on `usize` (a helper so the arithmetic stays `usize`-typed).
fn uadd(a: usize, b: usize) -> usize {
    a + b
}

/// `a + 1` on `usize`.
fn add1(a: usize) -> usize {
    a + 1
}

/// `v` shifted by `base`, except that the sentinel `0` stays `0`.
fn shifted(v: usize, base: usize) -> usize {
    if v == 0 {
        0
    } else {
        base + v
    }
}

/// A fresh heap with just the sentinel slot and an empty root forest.
pub fn binom_new() -> BinomHeap {
    let key: Vec<i64> = vec![0; 1];
    let degree: Vec<usize> = vec![0; 1];
    let parent: Vec<usize> = vec![0; 1];
    let child: Vec<usize> = vec![0; 1];
    let sibling: Vec<usize> = vec![0; 1];
    let roots: Vec<usize> = Vec::new();
    BinomHeap { key, degree, parent, child, sibling, roots, n: 0 }
}

/// Make `c` the first child of `p` (`p` and `c` must have the same degree).
/// The combined tree is rooted at `p` and has degree `degree[p] + 1`.
fn link_trees(mut h: BinomHeap, p: usize, c: usize) -> BinomHeap {
    let sib = h.child[p];
    let d = h.degree[p];
    let d1 = add1(d);
    h.sibling[c] = sib;
    h.child[p] = c;
    h.parent[c] = p;
    h.degree[p] = d1;
    h
}

/// Link two binomial trees of equal degree, rooted at `a` and `b`; the one
/// with the smaller key becomes the root.  Returns the updated heap and the
/// new root.
fn link_pair(h: BinomHeap, a: usize, b: usize) -> (BinomHeap, usize) {
    let ka = h.key[a];
    let kb = h.key[b];
    if kb < ka {
        let h2 = link_trees(h, b, a);
        (h2, b)
    } else {
        let h2 = link_trees(h, a, b);
        (h2, a)
    }
}

/// Carry the tree rooted at `x` up through the degree-indexed bucket array
/// `slot` until it reaches a free slot, then place it there.  Returns the
/// updated heap and buckets.
fn place(h: BinomHeap, slot: Vec<usize>, x: usize) -> (BinomHeap, Vec<usize>) {
    let d = h.degree[x];
    let occ = slot[d];
    if occ == 0 {
        let mut s2 = slot;
        s2[d] = x;
        (h, s2)
    } else {
        let mut s2 = slot;
        s2[d] = 0;
        let (h2, xr) = link_pair(h, x, occ);
        place(h2, s2, xr)
    }
}

/// Consolidate the root forest: repeatedly link pairs of equal-degree roots
/// (via a degree-indexed bucket array) until every degree occurs at most once.
/// The rebuilt root list is sorted by increasing degree.
fn consolidate(h: BinomHeap) -> BinomHeap {
    let mut hh = h;
    let cap = add1(hh.n);
    let mut slot: Vec<usize> = vec![0; cap];
    let mut roots: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < hh.roots.len() {
        let r = hh.roots[i];
        roots.push(r);
        i += 1;
    }
    let mut j: usize = 0;
    while j < roots.len() {
        let w = roots[j];
        let (h2, s2) = place(hh, slot, w);
        hh = h2;
        slot = s2;
        j += 1;
    }
    let mut newroots: Vec<usize> = Vec::new();
    let mut d: usize = 0;
    while d < slot.len() {
        let s = slot[d];
        if s != 0 {
            newroots.push(s);
        }
        d += 1;
    }
    hh.roots = newroots;
    hh
}

/// Splice `b`'s node pool into `h`'s, shifting every index of `b` by `h.n`.
fn splice(h: BinomHeap, b: BinomHeap) -> BinomHeap {
    let mut hh = h;
    let base = hh.n;
    let bn = b.n;
    let mut j: usize = 1;
    while j <= bn {
        let kk = b.key[j];
        hh.key.push(kk);
        let dg = b.degree[j];
        hh.degree.push(dg);
        let p = b.parent[j];
        let ps = shifted(p, base);
        hh.parent.push(ps);
        let c = b.child[j];
        let cs = shifted(c, base);
        hh.child.push(cs);
        let sb = b.sibling[j];
        let ss = shifted(sb, base);
        hh.sibling.push(ss);
        j += 1;
    }
    let bm = b.roots.len();
    let mut i: usize = 0;
    while i < bm {
        let r = b.roots[i];
        let rs = uadd(r, base);
        hh.roots.push(rs);
        i += 1;
    }
    hh.n = uadd(base, bn);
    hh
}

/// Union of two heaps (the key multiset of the result is the sum of the
/// inputs).
pub fn binom_merge(a: BinomHeap, b: BinomHeap) -> BinomHeap {
    let h = splice(a, b);
    consolidate(h)
}

/// Insert `key` into the heap.
pub fn binom_insert(h: BinomHeap, k: i64) -> BinomHeap {
    let mut hh = h;
    let idx = add1(hh.n);
    hh.key.push(k);
    hh.degree.push(0);
    hh.parent.push(0);
    hh.child.push(0);
    hh.sibling.push(0);
    hh.n = idx;
    hh.roots.push(idx);
    consolidate(hh)
}

/// The minimum stored key (the heap must be non-empty).
pub fn binom_find_min(h: &BinomHeap) -> i64 {
    let r0 = h.roots[0];
    let mut m = h.key[r0];
    let mut i: usize = 0;
    while i < h.roots.len() {
        let r = h.roots[i];
        let k = h.key[r];
        if k < m {
            m = k;
        }
        i += 1;
    }
    m
}

/// Remove one occurrence of the minimum key and return it together with the
/// updated heap.  The popped root's children are promoted to roots and the
/// forest is consolidated again.
pub fn binom_extract_min(h: BinomHeap) -> (i64, BinomHeap) {
    let mut hh = h;
    // find the root with the smallest key (the first one on ties)
    let mut mi: usize = 0;
    let mut i: usize = 0;
    while i < hh.roots.len() {
        let r = hh.roots[i];
        if mi == 0 {
            mi = r;
        } else {
            let k = hh.key[r];
            let km = hh.key[mi];
            if k < km {
                mi = r;
            }
        }
        i += 1;
    }
    let mk = hh.key[mi];
    // the other roots ...
    let mut newroots: Vec<usize> = Vec::new();
    let mut j: usize = 0;
    while j < hh.roots.len() {
        let r = hh.roots[j];
        if r != mi {
            newroots.push(r);
        }
        j += 1;
    }
    // ... plus the (now unlinked) children of the popped root
    let mut c = hh.child[mi];
    while c != 0 {
        let nx = hh.sibling[c];
        hh.sibling[c] = 0;
        hh.parent[c] = 0;
        newroots.push(c);
        c = nx;
    }
    hh.roots = newroots;
    let h2 = consolidate(hh);
    (mk, h2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Is `v` the root of a well-formed binomial tree of degree `d` (heap order
    /// included)?  Native-only; not translated.
    fn c_is_tree(h: &BinomHeap, v: usize, d: usize) -> bool {
        if h.degree[v] != d {
            return false;
        }
        if d == 0 {
            return h.child[v] == 0;
        }
        let mut c = h.child[v];
        let mut k = d;
        while k > 0 {
            if c == 0 {
                return false;
            }
            if h.parent[c] != v {
                return false;
            }
            if h.key[v] > h.key[c] {
                return false;
            }
            if !c_is_tree(h, c, k - 1) {
                return false;
            }
            c = h.sibling[c];
            k -= 1;
        }
        c == 0
    }

    /// The full representation invariant used by the spec: every root is a
    /// binomial tree of its recorded degree and the root degrees are distinct.
    fn c_invariant(h: &BinomHeap) -> bool {
        let mut i: usize = 0;
        while i < h.roots.len() {
            let r = h.roots[i];
            if !c_is_tree(h, r, h.degree[r]) {
                return false;
            }
            i += 1;
        }
        let mut i: usize = 0;
        while i < h.roots.len() {
            let mut j: usize = 0;
            while j < h.roots.len() {
                if i != j {
                    let di = h.degree[h.roots[i]];
                    let dj = h.degree[h.roots[j]];
                    if di == dj {
                        return false;
                    }
                }
                j += 1;
            }
            i += 1;
        }
        c_roots_ok(h) && c_forest_ok(h)
    }

    /// Roots must be nonzero, parentless, siblingless and inside the pool.
    fn c_roots_ok(h: &BinomHeap) -> bool {
        let mut i: usize = 0;
        while i < h.roots.len() {
            let r = h.roots[i];
            if r == 0 || r > h.n {
                return false;
            }
            if h.parent[r] != 0 || h.sibling[r] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    fn c_collect(h: &BinomHeap, v: usize, acc: &mut Vec<i64>) {
        acc.push(h.key[v]);
        let mut c = h.child[v];
        while c != 0 {
            c_collect(h, c, acc);
            c = h.sibling[c];
        }
    }

    /// All keys reachable from the root forest.
    fn c_all_keys(h: &BinomHeap) -> Vec<i64> {
        let mut acc: Vec<i64> = Vec::new();
        let mut i: usize = 0;
        while i < h.roots.len() {
            let r = h.roots[i];
            c_collect(h, r, &mut acc);
            i += 1;
        }
        acc
    }

    fn c_nodes(h: &BinomHeap, v: usize, acc: &mut Vec<usize>) {
        acc.push(v);
        let mut c = h.child[v];
        while c != 0 {
            c_nodes(h, c, acc);
            c = h.sibling[c];
        }
    }

    /// All node indices reachable from the root forest.
    fn c_forest_nodes(h: &BinomHeap) -> Vec<usize> {
        let mut acc: Vec<usize> = Vec::new();
        let mut i: usize = 0;
        while i < h.roots.len() {
            let r = h.roots[i];
            c_nodes(h, r, &mut acc);
            i += 1;
        }
        acc
    }

    /// The forest is a disjoint union of trees of `2 ^ degree` nodes each and
    /// lives inside the pool.
    fn c_forest_ok(h: &BinomHeap) -> bool {
        let nodes = c_forest_nodes(h);
        // inside the pool
        let mut i: usize = 0;
        while i < nodes.len() {
            if nodes[i] == 0 || nodes[i] > h.n {
                return false;
            }
            i += 1;
        }
        // pairwise disjoint (no node appears twice)
        let mut i: usize = 0;
        while i < nodes.len() {
            let mut j: usize = 0;
            while j < nodes.len() {
                if i != j && nodes[i] == nodes[j] {
                    return false;
                }
                j += 1;
            }
            i += 1;
        }
        // node count equals sum over roots of 2 ^ degree
        let mut s: usize = 0;
        let mut i: usize = 0;
        while i < h.roots.len() {
            let r = h.roots[i];
            let d = h.degree[r];
            let mut p: usize = 1;
            let mut k: usize = 0;
            while k < d {
                p *= 2;
                k += 1;
            }
            s += p;
            i += 1;
        }
        nodes.len() == s
    }

    fn c_sort(mut v: Vec<i64>) -> Vec<i64> {
        let n = v.len();
        let mut i: usize = 0;
        while i < n {
            let mut j: usize = 0;
            while j + 1 < n {
                let x = v[j];
                let y = v[j + 1];
                if x > y {
                    v[j] = y;
                    v[j + 1] = x;
                }
                j += 1;
            }
            i += 1;
        }
        v
    }

    fn c_min(v: &Vec<i64>) -> i64 {
        let mut m = v[0];
        let mut i: usize = 0;
        while i < v.len() {
            if v[i] < m {
                m = v[i];
            }
            i += 1;
        }
        m
    }

    fn c_build(keys: &Vec<i64>) -> BinomHeap {
        let mut h = binom_new();
        let mut i: usize = 0;
        while i < keys.len() {
            let k = keys[i];
            h = binom_insert(h, k);
            i += 1;
        }
        h
    }

    fn c_lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    #[test]
    fn insert_find_min() {
        let keys = vec![5i64, 3, 7, 2, 4, 9, 1, 8];
        let mut h = binom_new();
        let mut i: usize = 0;
        while i < keys.len() {
            let k = keys[i];
            h = binom_insert(h, k);
            assert!(c_invariant(&h));
            i += 1;
        }
        assert_eq!(binom_find_min(&h), 1);
        // extract everything; the sequence must be non-decreasing
        let mut prev: i64 = -1;
        let mut count: usize = 0;
        while count < keys.len() {
            let (k, h2) = binom_extract_min(h);
            h = h2;
            assert!(k >= prev);
            prev = k;
            count += 1;
            assert!(c_invariant(&h));
        }
        assert_eq!(prev, 9);
        assert_eq!(h.roots.len(), 0);
    }

    #[test]
    fn merge_and_duplicates() {
        let a = vec![10i64, 20, 30];
        let b = vec![5i64, 15, 25];
        let mut ha = binom_new();
        let mut i: usize = 0;
        while i < a.len() {
            let k = a[i];
            ha = binom_insert(ha, k);
            i += 1;
        }
        let mut hb = binom_new();
        let mut i: usize = 0;
        while i < b.len() {
            let k = b[i];
            hb = binom_insert(hb, k);
            i += 1;
        }
        let hm = binom_merge(ha, hb);
        assert!(c_invariant(&hm));
        assert_eq!(binom_find_min(&hm), 5);
        let (k1, h1) = binom_extract_min(hm);
        assert_eq!(k1, 5);
        let (k2, _h2) = binom_extract_min(h1);
        assert_eq!(k2, 10);
    }

    #[test]
    fn extract_min_removes_one_occurrence() {
        let keys = vec![3i64, 1, 3, 1, 3, 1];
        let mut h = binom_new();
        let mut i: usize = 0;
        while i < keys.len() {
            let k = keys[i];
            h = binom_insert(h, k);
            i += 1;
        }
        let mut n: usize = 0;
        while n < 3 {
            let before = c_all_keys(&h).len();
            let (k, h2) = binom_extract_min(h);
            h = h2;
            assert_eq!(k, 1);
            assert_eq!(c_all_keys(&h).len(), before - 1);
            assert!(c_invariant(&h));
            n += 1;
        }
        assert_eq!(binom_find_min(&h), 3);
        assert_eq!(c_all_keys(&h).len(), 3);
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 0x1234_5678_9abc_def0;
        let mut h = binom_new();
        let mut refs: Vec<i64> = Vec::new();
        let mut ops: usize = 0;
        while ops < 400 {
            let r = c_lcg(&mut state);
            let op = (r >> 5) % 4;
            if op < 2 {
                let k = ((r >> 10) % 50) as i64 - 25;
                h = binom_insert(h, k);
                refs.push(k);
            } else if op == 2 {
                // merge in a fresh multi-key heap
                let mut ks: Vec<i64> = Vec::new();
                let mut t: usize = 0;
                while t < 3 {
                    let rr = c_lcg(&mut state);
                    let kk = ((rr >> 7) % 50) as i64 - 25;
                    ks.push(kk);
                    refs.push(kk);
                    t += 1;
                }
                let hb = c_build(&ks);
                assert!(c_invariant(&hb));
                h = binom_merge(h, hb);
            } else if refs.len() > 0 {
                let (k, h2) = binom_extract_min(h);
                h = h2;
                let mn = c_min(&refs);
                assert_eq!(k, mn);
                let mut idx: usize = 0;
                while refs[idx] != mn {
                    idx += 1;
                }
                refs.remove(idx);
            }
            assert!(c_invariant(&h));
            assert_eq!(c_sort(c_all_keys(&h)), c_sort(refs.clone()));
            if refs.len() > 0 {
                assert_eq!(binom_find_min(&h), c_min(&refs));
            }
            ops += 1;
        }
    }
}
