//! van Emde Boas tree — `O(log log u)` set over a universe `u = 2^(2^m)`
//! (so `u ∈ {2, 4, 16, 256, ...}`; the universe must be a perfect square with
//! an integer square root that is itself a valid universe).
//!
//! A node of universe `u` stores its minimum and maximum explicitly and, when
//! `u ≥ 4`, splits the universe into `√u` *clusters*, each a vEB tree over the
//! low `√u` values; a `summary`, itself a vEB tree over the `√u` cluster
//! indices, records exactly the non-empty clusters.  This is the classic
//! recursive `√u` decomposition.
//!
//! Two structural conventions are load-bearing and are mirrored exactly by the
//! specification:
//!
//! * The `min` of a node is **not** stored recursively in the clusters — it is
//!   kept only at the node itself.  The `max` *is* stored in its cluster (when
//!   the node holds at least two elements).  Only nodes of universe `2` keep
//!   both endpoints locally (they have no clusters).
//! * An **empty** node is flagged by `empty`; its `min`/`max` fields are then
//!   meaningless.  `veb_successor` uses the pair `(found, value)` and returns
//!   `value = 0` whenever `found = false` — the documented sentinel.
//!
//! `Veb` and `Clusters` are mutually recursive pure data; `Box` is erased by
//! Aeneas, yielding a pair of mutually-inductive Lean types.

/// A vEB node: universe `u`, its square root `half = √u`, an emptiness flag,
/// the stored `min`/`max`, the `summary` over cluster indices and the list of
/// `√u` clusters.  `summary`/`clusters` are `None` only at the `u = 2` base.
pub struct Veb {
    pub u: usize,
    pub half: usize,
    pub empty: bool,
    pub min: usize,
    pub max: usize,
    pub summary: Option<Box<Veb>>,
    pub clusters: Option<Box<Clusters>>,
}

/// A cons-list holding the `√u` clusters of a node (the `i`-th cluster is the
/// `i`-th cell).  Each cell is a full vEB tree of universe `√u`.
pub struct Clusters {
    pub head: Veb,
    pub tail: Option<Box<Clusters>>,
}

// -- small arithmetic helpers ------------------------------------------------
//
// Every `usize` computation is routed through one of these so that the
// arithmetic operands are always function *parameters* (tracked by Aeneas as
// `usize`), never struct-field reads or `Vec` lengths (which Aeneas
// mis-classifies as `Int`).

fn add_u(a: usize, b: usize) -> usize {
    let r = a + b;
    r
}

fn sub_u(a: usize, b: usize) -> usize {
    let r = a - b;
    r
}

fn mul_u(a: usize, b: usize) -> usize {
    let r = a * b;
    r
}

fn div_u(a: usize, b: usize) -> usize {
    let r = a / b;
    r
}

fn eq_u(a: usize, b: usize) -> bool {
    if a == b {
        true
    } else {
        false
    }
}

fn lt_u(a: usize, b: usize) -> bool {
    if a < b {
        true
    } else {
        false
    }
}

fn gt_u(a: usize, b: usize) -> bool {
    if a > b {
        true
    } else {
        false
    }
}

/// `√u` for a valid universe `u = 2^(2^m)` with `u ≥ 4` (`u = 2` is never
/// passed here).  Computed by doubling from `2` while the square is too small.
fn sqrt_of(u: usize) -> usize {
    let mut r = 2;
    while mul_u(r, r) < u {
        r = mul_u(r, 2);
    }
    r
}

/// Build a list of `n` empty clusters each of universe `cu`.
fn make_clusters(n: usize, cu: usize) -> Option<Box<Clusters>> {
    if eq_u(n, 0) {
        None
    } else {
        let hd = veb_new(cu);
        let tl = make_clusters(sub_u(n, 1), cu);
        Some(Box::new(Clusters { head: hd, tail: tl }))
    }
}

/// Construct an empty vEB tree over universe `u`.
pub fn veb_new(u: usize) -> Veb {
    if eq_u(u, 2) {
        Veb {
            u: 2,
            half: 1,
            empty: true,
            min: 0,
            max: 0,
            summary: None,
            clusters: None,
        }
    } else {
        let h = sqrt_of(u);
        let sum = veb_new(h);
        let cl = make_clusters(h, h);
        Veb {
            u,
            half: h,
            empty: true,
            min: 0,
            max: 0,
            summary: Some(Box::new(sum)),
            clusters: cl,
        }
    }
}

/// A leaf of universe `2` holding the single element `x`.
fn leaf(x: usize) -> Veb {
    Veb {
        u: 2,
        half: 1,
        empty: false,
        min: x,
        max: x,
        summary: None,
        clusters: None,
    }
}

/// Make `v` empty while keeping its shape (its clusters are already empty).
fn clear(v: Veb) -> Veb {
    Veb {
        u: v.u,
        half: v.half,
        empty: true,
        min: 0,
        max: 0,
        summary: v.summary,
        clusters: v.clusters,
    }
}

// -- membership --------------------------------------------------------------

fn not_empty(v: &Veb) -> bool {
    if v.empty {
        false
    } else {
        true
    }
}

/// `l`-th element of the `i`-th cluster of the list.
fn cl_member(c: &Option<Box<Clusters>>, i: usize, l: usize) -> bool {
    match c {
        None => false,
        Some(cell) => {
            if eq_u(i, 0) {
                veb_member(&cell.head, l)
            } else {
                let i1 = sub_u(i, 1);
                cl_member(&cell.tail, i1, l)
            }
        }
    }
}

/// Is `x` stored in `v`?
pub fn veb_member(v: &Veb, x: usize) -> bool {
    if v.empty {
        return false;
    }
    if eq_u(x, v.min) {
        return true;
    }
    if eq_u(v.u, 2) {
        return eq_u(x, v.max);
    }
    let h = div_u(x, v.half);
    let l = sub_u(x, mul_u(h, v.half));
    cl_member(&v.clusters, h, l)
}

// -- min / max ---------------------------------------------------------------

/// `(non-empty?, minimum)` of `v` (the second component is `0` when empty).
pub fn veb_min(v: &Veb) -> (bool, usize) {
    if v.empty {
        (false, 0)
    } else {
        (true, v.min)
    }
}

/// `(non-empty?, maximum)` of `v` (the second component is `0` when empty).
pub fn veb_max(v: &Veb) -> (bool, usize) {
    if v.empty {
        (false, 0)
    } else {
        (true, v.max)
    }
}

fn veb_min_opt(s: &Option<Box<Veb>>) -> (bool, usize) {
    match s {
        None => (false, 0),
        Some(b) => veb_min(b),
    }
}

fn veb_max_opt(s: &Option<Box<Veb>>) -> (bool, usize) {
    match s {
        None => (false, 0),
        Some(b) => veb_max(b),
    }
}

fn veb_member_opt(s: &Option<Box<Veb>>, x: usize) -> bool {
    match s {
        None => false,
        Some(b) => veb_member(b, x),
    }
}

fn is_empty_opt(s: &Option<Box<Veb>>) -> bool {
    match s {
        None => true,
        Some(b) => b.empty,
    }
}

// -- successor ---------------------------------------------------------------

fn cl_successor(c: &Option<Box<Clusters>>, i: usize, l: usize) -> (bool, usize) {
    match c {
        None => (false, 0),
        Some(cell) => {
            if eq_u(i, 0) {
                veb_successor(&cell.head, l)
            } else {
                let i1 = sub_u(i, 1);
                cl_successor(&cell.tail, i1, l)
            }
        }
    }
}

fn cl_min(c: &Option<Box<Clusters>>, i: usize) -> (bool, usize) {
    match c {
        None => (false, 0),
        Some(cell) => {
            if eq_u(i, 0) {
                veb_min(&cell.head)
            } else {
                let i1 = sub_u(i, 1);
                cl_min(&cell.tail, i1)
            }
        }
    }
}

fn cl_max(c: &Option<Box<Clusters>>, i: usize) -> (bool, usize) {
    match c {
        None => (false, 0),
        Some(cell) => {
            if eq_u(i, 0) {
                veb_max(&cell.head)
            } else {
                let i1 = sub_u(i, 1);
                cl_max(&cell.tail, i1)
            }
        }
    }
}

fn veb_successor_opt(s: &Option<Box<Veb>>, x: usize) -> (bool, usize) {
    match s {
        None => (false, 0),
        Some(b) => veb_successor(b, x),
    }
}

/// Least element of `v` strictly greater than `x`; `(false, 0)` if there is
/// none (the second component is the documented sentinel `0`).
pub fn veb_successor(v: &Veb, x: usize) -> (bool, usize) {
    if v.empty {
        return (false, 0);
    }
    if lt_u(x, v.min) {
        return (true, v.min);
    }
    if eq_u(v.u, 2) {
        if lt_u(x, v.max) {
            return (true, v.max);
        }
        return (false, 0);
    }
    let h = div_u(x, v.half);
    let l = sub_u(x, mul_u(h, v.half));
    let (cf, cv) = cl_successor(&v.clusters, h, l);
    if cf {
        let base = mul_u(h, v.half);
        let r = add_u(base, cv);
        return (true, r);
    }
    let (sf, sj) = veb_successor_opt(&v.summary, h);
    if !sf {
        return (false, 0);
    }
    let (mf, mv) = cl_min(&v.clusters, sj);
    if !mf {
        return (false, 0);
    }
    let base = mul_u(sj, v.half);
    let r = add_u(base, mv);
    (true, r)
}

// -- insert ------------------------------------------------------------------

fn cl_insert(c: Option<Box<Clusters>>, i: usize, l: usize) -> Option<Box<Clusters>> {
    match c {
        None => None,
        Some(cell) => {
            let cellv = *cell;
            if eq_u(i, 0) {
                let nh = veb_insert(cellv.head, l);
                Some(Box::new(Clusters {
                    head: nh,
                    tail: cellv.tail,
                }))
            } else {
                let i1 = sub_u(i, 1);
                let nt = cl_insert(cellv.tail, i1, l);
                Some(Box::new(Clusters {
                    head: cellv.head,
                    tail: nt,
                }))
            }
        }
    }
}

fn veb_insert_opt(s: Option<Box<Veb>>, x: usize) -> Option<Box<Veb>> {
    match s {
        None => None,
        Some(b) => {
            let nb = veb_insert(*b, x);
            Some(Box::new(nb))
        }
    }
}

/// Insert `x` into `v` (`x` must satisfy `x < v.u`).  Inserting an element that
/// is already present leaves the stored set unchanged.
pub fn veb_insert(v: Veb, x: usize) -> Veb {
    if v.empty {
        return Veb {
            u: v.u,
            half: v.half,
            empty: false,
            min: x,
            max: x,
            summary: v.summary,
            clusters: v.clusters,
        };
    }
    if eq_u(x, v.min) {
        return v;
    }
    if eq_u(v.u, 2) {
        let nmin = if lt_u(x, v.min) { x } else { v.min };
        let nmax = if gt_u(x, v.max) { x } else { v.max };
        return Veb {
            u: 2,
            half: 1,
            empty: false,
            min: nmin,
            max: nmax,
            summary: None,
            clusters: None,
        };
    }
    let uu = v.u;
    let half = v.half;
    let oldmin = v.min;
    let oldmax = v.max;
    let mut nmin = oldmin;
    let mut y = x;
    if lt_u(x, oldmin) {
        // `x` becomes the new node minimum; the old minimum is inserted
        // recursively (so that the node minimum is never stored in a cluster).
        nmin = x;
        y = oldmin;
    }
    let h = div_u(y, half);
    let l = sub_u(y, mul_u(h, half));
    let was_nonempty = veb_member_opt(&v.summary, h);
    let new_clusters = cl_insert(v.clusters, h, l);
    let new_summary = if was_nonempty {
        v.summary
    } else {
        veb_insert_opt(v.summary, h)
    };
    let nmax = if gt_u(y, oldmax) { y } else { oldmax };
    Veb {
        u: uu,
        half,
        empty: false,
        min: nmin,
        max: nmax,
        summary: new_summary,
        clusters: new_clusters,
    }
}

// -- delete ------------------------------------------------------------------

fn cl_nonempty(c: &Option<Box<Clusters>>, i: usize) -> bool {
    match c {
        None => false,
        Some(cell) => {
            if eq_u(i, 0) {
                not_empty(&cell.head)
            } else {
                let i1 = sub_u(i, 1);
                cl_nonempty(&cell.tail, i1)
            }
        }
    }
}

fn cl_delete(c: Option<Box<Clusters>>, i: usize, l: usize) -> Option<Box<Clusters>> {
    match c {
        None => None,
        Some(cell) => {
            let cellv = *cell;
            if eq_u(i, 0) {
                let nh = veb_delete(cellv.head, l);
                Some(Box::new(Clusters {
                    head: nh,
                    tail: cellv.tail,
                }))
            } else {
                let i1 = sub_u(i, 1);
                let nt = cl_delete(cellv.tail, i1, l);
                Some(Box::new(Clusters {
                    head: cellv.head,
                    tail: nt,
                }))
            }
        }
    }
}

fn veb_delete_opt(s: Option<Box<Veb>>, x: usize) -> Option<Box<Veb>> {
    match s {
        None => None,
        Some(b) => {
            let nb = veb_delete(*b, x);
            Some(Box::new(nb))
        }
    }
}

/// Delete `x` from `v` (deleting an absent element is a no-op).
pub fn veb_delete(v: Veb, x: usize) -> Veb {
    if v.empty {
        return v;
    }
    if eq_u(v.min, v.max) {
        // singleton: removing its only element empties the node
        if eq_u(x, v.min) {
            return clear(v);
        }
        return v;
    }
    if eq_u(v.u, 2) {
        if eq_u(x, v.min) {
            return leaf(v.max);
        }
        if eq_u(x, v.max) {
            return leaf(v.min);
        }
        return v;
    }
    let uu = v.u;
    let half = v.half;
    let oldmin = v.min;
    let oldmax = v.max;
    // The element actually withdrawn from the clusters.  When `x` is the node
    // minimum the next smallest element is promoted to `min` and removed from
    // its cluster instead.
    let mut xdel = x;
    if eq_u(x, oldmin) {
        let (jf, j) = veb_min_opt(&v.summary);
        let (mf, mv) = cl_min(&v.clusters, j);
        let base = mul_u(j, half);
        xdel = add_u(base, mv);
        let _ = jf;
        let _ = mf;
    }
    let nmin = if eq_u(x, oldmin) { xdel } else { oldmin };
    let h = div_u(xdel, half);
    let l = sub_u(xdel, mul_u(h, half));
    let new_clusters = cl_delete(v.clusters, h, l);
    let hnon = cl_nonempty(&new_clusters, h);
    let new_summary = if hnon {
        v.summary
    } else {
        veb_delete_opt(v.summary, h)
    };
    let mut nmax = oldmax;
    if eq_u(xdel, oldmax) {
        if is_empty_opt(&new_summary) {
            // only the minimum is left
            nmax = nmin;
        } else {
            let (jf2, j2) = veb_max_opt(&new_summary);
            let (mf2, mv2) = cl_max(&new_clusters, j2);
            let base2 = mul_u(j2, half);
            nmax = add_u(base2, mv2);
            let _ = jf2;
            let _ = mf2;
        }
    }
    Veb {
        u: uu,
        half,
        empty: false,
        min: nmin,
        max: nmax,
        summary: new_summary,
        clusters: new_clusters,
    }
}

// ---------------------------------------------------------------------------
// Heavy differential tests (native only — this module is not translated).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Elements stored *in the clusters* of `v`, shifted to global coords.
    fn t_cl_elems(v: &Veb, acc: &mut Vec<usize>) {
        let mut c = &v.clusters;
        let mut i: usize = 0;
        while let Some(cell) = c {
            let base = i * v.half;
            let mut sub: Vec<usize> = Vec::new();
            t_elems(&cell.head, &mut sub);
            let mut k: usize = 0;
            while k < sub.len() {
                acc.push(base + sub[k]);
                k += 1;
            }
            c = &cell.tail;
            i += 1;
        }
    }

    /// All elements stored in `v` (the node `min`, the leaf `max`, and every
    /// cluster element).
    fn t_elems(v: &Veb, acc: &mut Vec<usize>) {
        if v.empty {
            return;
        }
        acc.push(v.min);
        if v.u == 2 {
            if v.max != v.min {
                acc.push(v.max);
            }
            return;
        }
        t_cl_elems(v, acc);
    }

    fn t_elems_of(v: &Veb) -> Vec<usize> {
        let mut acc: Vec<usize> = Vec::new();
        t_elems(v, &mut acc);
        acc
    }

    fn t_cl_len(c: &Option<Box<Clusters>>) -> usize {
        match c {
            None => 0,
            Some(cell) => 1 + t_cl_len(&cell.tail),
        }
    }

    /// Mirror of the specification invariant, checked on native values.
    fn t_inv(v: &Veb) -> bool {
        if v.empty {
            return true;
        }
        // universe well-formedness
        if v.u == 2 {
            if v.clusters.is_some() || v.summary.is_some() {
                return false;
            }
            return v.min < 2 && v.max < 2;
        }
        if v.half * v.half != v.u || v.half < 2 {
            return false;
        }
        if v.min >= v.u || v.max >= v.u {
            return false;
        }
        if t_cl_len(&v.clusters) != v.half {
            return false;
        }
        // min / max correctness
        let els = t_elems_of(v);
        let mut i: usize = 0;
        let mut lo = v.u;
        let mut hi = 0;
        while i < els.len() {
            if els[i] < lo {
                lo = els[i];
            }
            if els[i] > hi {
                hi = els[i];
            }
            i += 1;
        }
        if lo != v.min || hi != v.max {
            return false;
        }
        // the node minimum is not stored recursively in the clusters
        let mut clels: Vec<usize> = Vec::new();
        t_cl_elems(v, &mut clels);
        let mut i2: usize = 0;
        while i2 < clels.len() {
            if clels[i2] == v.min {
                return false;
            }
            i2 += 1;
        }
        // summary agrees with the non-empty clusters, and has the right universe
        let mut sumels: Vec<usize> = Vec::new();
        match &v.summary {
            None => return false,
            Some(s) => t_elems(s, &mut sumels),
        }
        if let Some(s) = &v.summary {
            if s.u != v.half {
                return false;
            }
        }
        let mut nonempty: Vec<usize> = Vec::new();
        let mut c2 = &v.clusters;
        let mut idx2: usize = 0;
        while let Some(cell) = c2 {
            if !cell.head.empty {
                nonempty.push(idx2);
            }
            if cell.head.u != v.half {
                return false;
            }
            if !t_inv(&cell.head) {
                return false;
            }
            c2 = &cell.tail;
            idx2 += 1;
        }
        if sumels != nonempty {
            return false;
        }
        match &v.summary {
            None => false,
            Some(s) => t_inv(s),
        }
    }

    /// Brute-force successor over a bitmap.
    fn bf_succ(bits: &Vec<bool>, x: usize) -> (bool, usize) {
        let mut i = x + 1;
        while i < bits.len() {
            if bits[i] {
                return (true, i);
            }
            i += 1;
        }
        (false, 0)
    }

    fn bf_min(bits: &Vec<bool>) -> (bool, usize) {
        let mut i: usize = 0;
        while i < bits.len() {
            if bits[i] {
                return (true, i);
            }
            i += 1;
        }
        (false, 0)
    }

    fn bf_max(bits: &Vec<bool>) -> (bool, usize) {
        let mut i = bits.len();
        while i > 0 {
            i -= 1;
            if bits[i] {
                return (true, i);
            }
        }
        (false, 0)
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    #[test]
    fn basic_insert_member() {
        let mut t = veb_new(16);
        t = veb_insert(t, 3);
        t = veb_insert(t, 11);
        t = veb_insert(t, 7);
        assert!(veb_member(&t, 3));
        assert!(veb_member(&t, 11));
        assert!(veb_member(&t, 7));
        assert!(!veb_member(&t, 0));
        assert!(!veb_member(&t, 15));
        assert!(t_inv(&t));
        let (a, m) = veb_min(&t);
        assert!(a && m == 3);
        let (b, mx) = veb_max(&t);
        assert!(b && mx == 11);
    }

    #[test]
    fn successor_chain() {
        let mut t = veb_new(16);
        let mut i: usize = 0;
        while i < 16 {
            if i % 3 == 0 {
                t = veb_insert(t, i);
            }
            i += 1;
        }
        let mut j: usize = 0;
        while j < 16 {
            let (f, s) = veb_successor(&t, j);
            let (bf, bs) = bf_succ(&vec![
                true, false, false, true, false, false, true, false, false, true, false, false,
                true, false, false, true,
            ], j);
            assert_eq!(f, bf);
            assert_eq!(s, bs);
            j += 1;
        }
        assert!(t_inv(&t));
    }

    #[test]
    fn randomized_differential() {
        let u: usize = 16;
        let mut state: u64 = 0x1234_5678_9abc_def0;
        let mut t = veb_new(u);
        let mut bits: Vec<bool> = vec![false; u];
        let mut step: usize = 0;
        while step < 3000 {
            let r = lcg(&mut state);
            let x = ((r >> 7) % (u as u64)) as usize;
            if r % 3 == 0 {
                t = veb_delete(t, x);
                bits[x] = false;
            } else {
                t = veb_insert(t, x);
                bits[x] = true;
            }
            assert!(t_inv(&t));
            // full membership / successor / min / max differential check
            let mut y: usize = 0;
            while y < u {
                assert_eq!(veb_member(&t, y), bits[y]);
                let (f, s) = veb_successor(&t, y);
                let (bf, bs) = bf_succ(&bits, y);
                assert_eq!(f, bf);
                assert_eq!(s, bs);
                y += 1;
            }
            let (ma, mv) = veb_min(&t);
            let (ba, bv) = bf_min(&bits);
            assert_eq!(ma, ba);
            assert_eq!(mv, bv);
            let (xa, xv) = veb_max(&t);
            let (ca, cv) = bf_max(&bits);
            assert_eq!(xa, ca);
            assert_eq!(xv, cv);
            step += 1;
        }
    }

    #[test]
    fn all_of_universe() {
        let u: usize = 16;
        let mut t = veb_new(u);
        let mut i: usize = 0;
        while i < u {
            t = veb_insert(t, i);
            assert!(t_inv(&t));
            i += 1;
        }
        i = 0;
        while i < u {
            assert!(veb_member(&t, i));
            i += 1;
        }
        // successors: i -> i+1, and 15 -> none
        i = 0;
        while i + 1 < u {
            let (f, s) = veb_successor(&t, i);
            assert!(f && s == i + 1);
            i += 1;
        }
        let (f, s) = veb_successor(&t, 15);
        assert!(!f && s == 0);
        // delete everything from the front
        i = 0;
        while i < u {
            t = veb_delete(t, i);
            assert!(t_inv(&t));
            let (a, _) = veb_min(&t);
            let expect_min = i + 1 < u;
            assert_eq!(a, expect_min);
            i += 1;
        }
        assert!(t.empty);
    }

    #[test]
    fn universe_4_and_2() {
        let mut t = veb_new(4);
        t = veb_insert(t, 2);
        t = veb_insert(t, 0);
        t = veb_insert(t, 3);
        assert!(t_inv(&t));
        assert!(veb_member(&t, 0) && veb_member(&t, 2) && veb_member(&t, 3));
        assert!(!veb_member(&t, 1));
        let (f, s) = veb_successor(&t, 0);
        assert!(f && s == 2);
        let (f2, s2) = veb_successor(&t, 2);
        assert!(f2 && s2 == 3);
        t = veb_delete(t, 0);
        let (a, m) = veb_min(&t);
        assert!(a && m == 2);
        assert!(t_inv(&t));

        let mut b = veb_new(2);
        b = veb_insert(b, 1);
        assert!(veb_member(&b, 1) && !veb_member(&b, 0));
        let (bf, bs) = veb_successor(&b, 0);
        assert!(bf && bs == 1);
        b = veb_delete(b, 1);
        assert!(b.empty);
    }
}
