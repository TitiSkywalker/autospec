//! Fibonacci heap — a meldable priority queue.
//!
//! ## Representation
//!
//! Every node lives at a 1-based index of six parallel arrays (`0` is the
//! null / "no node" sentinel): `key`, `degree`, `parent`, `child`, `next`
//! (the sibling pointer) and `marked`.  A heap is those six arrays plus the
//! handle `min` (the index of a minimum root, `0` when the heap is empty) and
//! the allocation counter `used` (the number of slots ever handed out).
//! Both the root list and every child list are *circular* singly linked lists
//! threaded through `next`; a node is a root exactly when it is in the root
//! list, which is detected by `parent == 0`.
//!
//! ## Operations
//!
//! * `fib_insert`   — allocate a node and splice it into the root list;
//! * `fib_find_min` — the key of the root the handle `min` points at;
//! * `fib_merge`    — meld two heaps: the nodes of the second are renumbered
//!                    into the arena of the first (a flat-array artefact) and
//!                    the two circular root lists are spliced;
//! * `fib_extract_min` — cut the minimum root out, splice its children into the
//!                    root list, and consolidate the root list pairwise by
//!                    degree (the Fredman–Tarjan linking step).
//!
//! Consolidation only ever links two roots of the *same* degree `d`, and the
//! survivor of a link keeps its degree while the loser becomes degree-`d`
//! child number `d+1`.  Hence every node of degree `k` has children of degrees
//! `k-1, k-2, …, 0` — the *linking invariant*, from which the rank bound
//! `F_{k+2} ≤ #nodes in the subtree` follows by induction.
//!
//! `marked` is part of the node record and is maintained, but no `decrease_key`
//! is provided, so no node is ever marked (cascading cuts are not needed).
//!
//! Slots are never recycled: `used` is the number of slots ever handed out, so
//! the arena of a heap must be sized for the total number of insertions it will
//! ever see (`fib_new(n)` leaves room for `n` nodes).  A node removed by
//! `fib_extract_min` simply becomes unreachable, which is exactly how the
//! specification views the heap (the nodes are the slots reachable from `min`).

/// `a + 1`, routed through a helper so the arithmetic is performed on tracked
/// `usize` parameters rather than on a field access.
fn u_inc(a: usize) -> usize {
    a + 1
}

/// `a + b`.
fn u_add(a: usize, b: usize) -> usize {
    a + b
}

/// Renumber the node index `x` of a secondary arena by the slot offset `off`;
/// the null sentinel `0` is left alone.
fn shift(x: usize, off: usize) -> usize {
    if x == 0 { 0 } else { x + off }
}

/// A Fibonacci heap over the slots `1 .. used` of its arrays (slot `0` is the
/// null sentinel, so each array has room for one more entry than the heap has
/// nodes).
pub struct FibHeap {
    pub key: Vec<i64>,
    pub degree: Vec<usize>,
    pub parent: Vec<usize>,
    pub child: Vec<usize>,
    pub next: Vec<usize>,
    pub marked: Vec<usize>,
    pub min: usize,
    pub used: usize,
}

/// An empty heap with room for `n` nodes.
pub fn fib_new(n: usize) -> FibHeap {
    let size = n + 1;
    let key: Vec<i64> = vec![0; size];
    let degree: Vec<usize> = vec![0; size];
    let parent: Vec<usize> = vec![0; size];
    let child: Vec<usize> = vec![0; size];
    let next: Vec<usize> = vec![0; size];
    let marked: Vec<usize> = vec![0; size];
    FibHeap { key, degree, parent, child, next, marked, min: 0, used: 0 }
}

/// The last node of the circular list starting at `start`, i.e. the node whose
/// `next` points back at `start` (`0` when `start` is `0`).
fn list_last(h: &FibHeap, start: usize) -> usize {
    let mut w = start;
    while h.next[w] != start {
        w = h.next[w];
    }
    w
}

/// Add the singleton root `y` (its `next` is overwritten) to the root list,
/// updating the minimum handle.
fn add_root(h: FibHeap, y: usize) -> FibHeap {
    let mut h = h;
    let mn = h.min;
    if mn == 0 {
        h.min = y;
        h.next[y] = y;
    } else {
        let nx = h.next[mn];
        h.next[y] = nx;
        h.next[mn] = y;
        let ky = h.key[y];
        let km = h.key[mn];
        if ky < km {
            h.min = y;
        }
    }
    h
}

/// Splice the circular list starting at `b` into the circular list starting at
/// `a`, directly after `a` (`a` and `b` must both be non-zero).
fn splice_after(h: FibHeap, a: usize, b: usize) -> FibHeap {
    let mut h = h;
    if b != 0 {
        let blast = list_last(&h, b);
        let anext = h.next[a];
        h.next[blast] = anext;
        h.next[a] = b;
    }
    h
}

/// Insert the key `k` as a new root.  Returns the heap with one more node.
pub fn fib_insert(h: FibHeap, k: i64) -> FibHeap {
    let mut h = h;
    let x = u_inc(h.used);
    h.used = x;
    h.key[x] = k;
    h.degree[x] = 0;
    h.parent[x] = 0;
    h.child[x] = 0;
    h.next[x] = x;
    h.marked[x] = 0;
    add_root(h, x)
}

/// The minimum key, or `None` when the heap is empty.
pub fn fib_find_min(h: &FibHeap) -> Option<i64> {
    let mn = h.min;
    if mn == 0 {
        None
    } else {
        Some(h.key[mn])
    }
}

/// Make the root `y` a child of the root `x`.  The two must currently have the
/// same degree, which is what keeps the children degrees `k-1, …, 0`.
fn fib_link(h: FibHeap, y: usize, x: usize) -> FibHeap {
    let mut h = h;
    let c = h.child[x];
    if c == 0 {
        h.child[x] = y;
        h.next[y] = y;
    } else {
        let cn = h.next[c];
        h.next[y] = cn;
        h.next[c] = y;
    }
    h.parent[y] = x;
    h.marked[y] = 0;
    let dx = h.degree[x];
    let d1 = u_inc(dx);
    h.degree[x] = d1;
    h
}

/// Consolidate the single root `x` into the degree table `a`: while the slot
/// `a[degree x]` is occupied, link `x` with the occupant (the larger key
/// becomes a child of the smaller) and retry one degree higher.  Returns the
/// updated heap, the updated table, and the root that finally occupies the
/// slot.
fn table_add(h: FibHeap, a: Vec<usize>, x: usize) -> (FibHeap, Vec<usize>, usize) {
    let mut h = h;
    let mut a = a;
    let mut cur = x;
    let mut d = h.degree[x];
    let mut slot = a[d];
    while slot != 0 {
        let kc = h.key[cur];
        let ks = h.key[slot];
        let cw = kc > ks;
        // the root with the larger key becomes the child of the other one
        let big = if cw { cur } else { slot };
        let small = if cw { slot } else { cur };
        let h2 = fib_link(h, big, small);
        h = h2;
        a[d] = 0;
        cur = small;
        d = u_inc(d);
        slot = a[d];
    }
    a[d] = cur;
    (h, a, cur)
}

/// Make the children of `z` roots (their parent pointers are cleared) and
/// splice their chain in front of the `0`-terminated root chain `rest`.
fn detach_children(h: FibHeap, z: usize, rest: usize) -> FibHeap {
    let mut h = h;
    let c0 = h.child[z];
    if c0 != 0 {
        let cl = list_last(&h, c0);
        h.next[cl] = 0;
        let mut w = c0;
        while w != 0 {
            h.parent[w] = 0;
            w = h.next[w];
        }
        h.next[cl] = rest;
        h.child[z] = 0;
    }
    h
}

/// Rebuild the root list from the `0`-terminated chain `head` of roots, which
/// have already been pairwise linked by degree (see `table_add`).
fn consolidate(h: FibHeap, head: usize) -> FibHeap {
    let mut h = h;
    let tsize = u_add(h.used, 1);
    let mut a: Vec<usize> = vec![0; tsize];
    let mut w = head;
    while w != 0 {
        let nw = h.next[w];
        let (h2, a2, _cur) = table_add(h, a, w);
        h = h2;
        a = a2;
        w = nw;
    }
    h.min = 0;
    let n = a.len();
    let mut j: usize = 0;
    while j < n {
        let y = a[j];
        if y != 0 {
            let h2 = add_root(h, y);
            h = h2;
        }
        j += 1;
    }
    h
}

/// Remove the root `z` from the heap: cut `z` out of the root list, splice its
/// children in, and consolidate.
fn extract_root(h0: FibHeap, z: usize) -> FibHeap {
    let r0 = h0.next[z];
    let rl = list_last(&h0, z);
    let mut h = h0;
    h.next[rl] = 0;
    let r1 = if r0 == z { 0 } else { r0 };
    let c0 = h.child[z];
    let head = if c0 == 0 { r1 } else { c0 };
    let h1 = detach_children(h, z, r1);
    consolidate(h1, head)
}

/// Remove and return the minimum key together with the resulting heap; returns
/// `None` when the heap was empty.
pub fn fib_extract_min(h: FibHeap) -> (FibHeap, Option<i64>) {
    let z = h.min;
    if z == 0 {
        (h, None)
    } else {
        let k = h.key[z];
        let h2 = extract_root(h, z);
        (h2, Some(k))
    }
}

/// Meld `b` into `a`: the nodes of `b` are renumbered into the arena of `a`,
/// the two circular root lists are spliced, and the minimum handle is updated.
/// The result holds exactly the keys of both heaps; `a` must have room for the
/// nodes of `b`.
pub fn fib_merge(a: FibHeap, b: FibHeap) -> FibHeap {
    let mut h = a;
    let off = h.used;
    let m = b.used;
    let mut i: usize = 1;
    while i <= m {
        let p = u_add(off, i);
        let kb = b.key[i];
        h.key[p] = kb;
        let db = b.degree[i];
        h.degree[p] = db;
        let pb = b.parent[i];
        let npb = shift(pb, off);
        h.parent[p] = npb;
        let cb = b.child[i];
        let ncb = shift(cb, off);
        h.child[p] = ncb;
        let nb = b.next[i];
        let nnb = shift(nb, off);
        h.next[p] = nnb;
        let mb = b.marked[i];
        h.marked[p] = mb;
        i += 1;
    }
    let nu = u_add(off, m);
    h.used = nu;
    let bmin = shift(b.min, off);
    let amin = h.min;
    if amin == 0 {
        h.min = bmin;
    } else {
        if bmin != 0 {
            let h2 = splice_after(h, amin, bmin);
            h = h2;
            let km = h.key[amin];
            let kb = h.key[bmin];
            if kb < km {
                h.min = bmin;
            }
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    /// The children of `v`, in `next`-order.
    fn children_of(h: &FibHeap, v: usize) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        let c0 = h.child[v];
        if c0 != 0 {
            let mut w = c0;
            loop {
                out.push(w);
                w = h.next[w];
                if w == c0 {
                    break;
                }
            }
        }
        out
    }

    /// The roots, in `next`-order (`min` first).
    fn roots_of(h: &FibHeap) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        let z = h.min;
        if z != 0 {
            let mut w = z;
            loop {
                out.push(w);
                w = h.next[w];
                if w == z {
                    break;
                }
            }
        }
        out
    }

    /// Every slot reachable from the root list (following child/sibling links).
    fn reachable(h: &FibHeap) -> Vec<usize> {
        let n = h.key.len();
        let mut seen: Vec<usize> = vec![0; n];
        let mut out: Vec<usize> = Vec::new();
        let mut stack: Vec<usize> = roots_of(h);
        while !stack.is_empty() {
            let v = stack.pop().unwrap();
            if seen[v] == 1 {
                continue;
            }
            seen[v] = 1;
            out.push(v);
            for c in children_of(h, v) {
                stack.push(c);
            }
        }
        out
    }

    /// The multiset of live keys, in no particular order.
    fn live_keys(h: &FibHeap) -> Vec<i64> {
        let mut out: Vec<i64> = Vec::new();
        for v in reachable(h) {
            out.push(h.key[v]);
        }
        out
    }

    fn sorted(mut a: Vec<i64>) -> Vec<i64> {
        let n = a.len();
        let mut i: usize = 0;
        while i < n {
            let mut j: usize = 0;
            while j + 1 < n {
                let x = a[j];
                let y = a[j + 1];
                if x > y {
                    a[j] = y;
                    a[j + 1] = x;
                }
                j += 1;
            }
            i += 1;
        }
        a
    }

    /// The number of nodes in the subtree rooted at `v`.
    fn subtree_size(h: &FibHeap, v: usize) -> usize {
        let mut c: usize = 1;
        for ch in children_of(h, v) {
            c += subtree_size(h, ch);
        }
        c
    }

    /// `F_n` (`F_0 = 0`, `F_1 = 1`) — the Fibonacci numbers of the rank bound.
    fn small_fib(n: usize) -> usize {
        let mut a: usize = 0;
        let mut b: usize = 1;
        let mut i: usize = 0;
        while i < n {
            let t = a + b;
            a = b;
            b = t;
            i += 1;
        }
        a
    }

    /// The heap invariant, checked by brute force:
    ///  * the root list is circular and every root has `parent == 0`;
    ///  * heap order: a parent's key is `≤` its children's keys;
    ///  * `degree[v]` counts the children of `v`;
    ///  * the *linking invariant*: the child degrees of a degree-`k` node are
    ///    exactly `k-1, k-2, …, 0`;
    ///  * a non-root node is in its parent's child list, and the parent chain
    ///    leads to a root without cycling;
    ///  * the handle `min` points at a minimum root;
    ///  * every slot is used at most once (no node is its own descendant).
    fn check(h: &FibHeap, tag: &str) {
        let n = h.key.len();
        assert!(n >= 1, "{}: empty arena", tag);
        let nodes = reachable(h);
        let mut live: Vec<usize> = vec![0; n];
        for &v in &nodes {
            assert!(v >= 1 && v < n, "{}: bad slot {}", tag, v);
            // the arena discipline: a live node is a slot that was handed out
            assert!(v <= h.used, "{}: live node {} above used {}", tag, v, h.used);
            assert_eq!(live[v], 0, "{}: node {} reached twice (cycle)", tag, v);
            live[v] = 1;
        }
        let roots = roots_of(h);
        let mut is_root: Vec<usize> = vec![0; n];
        for &r in &roots {
            assert_eq!(h.parent[r], 0, "{}: root {} has a parent", tag, r);
            is_root[r] = 1;
        }
        // the min handle
        if nodes.is_empty() {
            assert_eq!(h.min, 0, "{}: empty heap with min {}", tag, h.min);
        } else {
            assert!(h.min != 0, "{}: non-empty heap without min", tag);
            assert_eq!(is_root[h.min], 1, "{}: min is not a root", tag);
            let mut mn: i64 = h.key[h.min];
            for &v in &nodes {
                let k = h.key[v];
                if k < mn {
                    mn = k;
                }
            }
            assert_eq!(h.key[h.min], mn, "{}: min handle is not minimal", tag);
        }
        // per-node structure
        for &v in &nodes {
            let cs = children_of(h, v);
            // a node either is a root, or sits in the child list of its parent
            if is_root[v] == 1 {
                assert_eq!(h.parent[v], 0, "{}: root {} has a parent", tag, v);
            } else {
                assert!(h.parent[v] != 0, "{}: non-root {} without parent", tag, v);
                assert_eq!(live[h.parent[v]], 1, "{}: {} hangs off a dead slot", tag, v);
                // ... and it really is in its parent's child list
                assert!(
                    children_of(h, h.parent[v]).contains(&v),
                    "{}: {} is not in the child list of its parent",
                    tag,
                    v
                );
            }
            // a parentless node is on the root cycle through `min`
            if h.parent[v] == 0 {
                assert_eq!(is_root[v], 1, "{}: {} is not on the root cycle", tag, v);
            }
            // `degree` counts the children
            assert_eq!(cs.len(), h.degree[v], "{}: degree of {} wrong", tag, v);
            // no node is ever marked (no decrease_key is implemented)
            assert_eq!(h.marked[v], 0, "{}: node {} is marked", tag, v);
            let k = h.key[v];
            for &c in &cs {
                // heap order
                assert!(k <= h.key[c], "{}: heap order at {} -> {}", tag, v, c);
                // parent link consistency
                assert_eq!(h.parent[c], v, "{}: parent of {} wrong", tag, c);
                // a child of a live node is live
                assert_eq!(live[c], 1, "{}: {} has a dead child {}", tag, v, c);
            }
            // linking invariant: the child degrees are exactly `0 .. degree-1`
            let mut degs: Vec<usize> = Vec::new();
            for &c in &cs {
                degs.push(h.degree[c]);
            }
            let m = degs.len();
            let mut i: usize = 0;
            while i < m {
                let mut j: usize = 0;
                while j + 1 < m {
                    if degs[j] > degs[j + 1] {
                        let x = degs[j];
                        degs[j] = degs[j + 1];
                        degs[j + 1] = x;
                    }
                    j += 1;
                }
                i += 1;
            }
            let mut d: usize = 0;
            while d < m {
                assert_eq!(degs[d], d, "{}: child degrees of {} are not 0..k-1", tag, v);
                d += 1;
            }
            // the *conclusion* of the rank bound, for the record
            assert!(
                small_fib(h.degree[v] + 2) <= subtree_size(h, v),
                "{}: rank bound violated at {}",
                tag,
                v
            );
        }
        // parent chains reach a root, without cycling
        for &v in &nodes {
            let mut cur = v;
            let mut steps: usize = 0;
            while h.parent[cur] != 0 {
                assert!(steps <= n, "{}: parent cycle at {}", tag, v);
                cur = h.parent[cur];
                steps += 1;
            }
            assert_eq!(is_root[cur], 1, "{}: {} hangs off a non-root", tag, v);
        }
    }

    #[test]
    fn empty_heap() {
        let h = fib_new(4);
        check(&h, "empty");
        assert!(fib_find_min(&h).is_none());
        let (h2, k) = fib_extract_min(h);
        check(&h2, "empty after extract");
        assert!(k.is_none());
        assert!(live_keys(&h2).is_empty());
    }

    #[test]
    fn single_insert_extract() {
        let h = fib_new(4);
        let h = fib_insert(h, 42);
        check(&h, "one");
        assert_eq!(fib_find_min(&h), Some(42));
        let (h2, k) = fib_extract_min(h);
        check(&h2, "one out");
        assert_eq!(k, Some(42));
        assert!(live_keys(&h2).is_empty());
    }

    #[test]
    fn extract_min_is_sorted_and_drains() {
        let mut h = fib_new(20);
        for k in [5i64, 3, 9, 1, 7, 3, 8, 0, -2, 6] {
            h = fib_insert(h, k);
            check(&h, "insert");
        }
        let mut out: Vec<i64> = Vec::new();
        let mut left = sorted(live_keys(&h));
        while !left.is_empty() {
            let want = left[0];
            let (h2, k) = fib_extract_min(h);
            h = h2;
            check(&h, "extract");
            // the returned key is the minimum of what was stored ...
            assert_eq!(k, Some(want));
            out.push(want);
            // ... and exactly one occurrence of it is gone
            left.remove(0);
            assert_eq!(sorted(live_keys(&h)), left);
        }
        assert_eq!(out, vec![-2, 0, 1, 3, 3, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn merge_agrees_with_key_multiset() {
        let mut a = fib_new(30);
        let mut b = fib_new(30);
        a = fib_insert(a, 10);
        a = fib_insert(a, 4);
        b = fib_insert(b, 7);
        b = fib_insert(b, 2);
        b = fib_insert(b, 9);
        check(&a, "a");
        check(&b, "b");
        let mut ka = live_keys(&a);
        let mut kb = live_keys(&b);
        let c = fib_merge(a, b);
        check(&c, "merge");
        ka.append(&mut kb);
        assert_eq!(sorted(live_keys(&c)), sorted(ka));
        assert_eq!(fib_find_min(&c), Some(2));
    }

    #[test]
    fn randomized_differential() {
        // Both arenas hold every slot ever allocated (isolated slots that were
        // extracted are never handed out again), so the capacity is the total
        // number of insertions.
        let cap = 120;
        let mut state: u64 = 20240921;
        let mut a = fib_new(cap);
        let mut b = fib_new(cap);
        let mut ka: Vec<i64> = Vec::new();
        let mut kb: Vec<i64> = Vec::new();
        let mut maxdeg: usize = 0;
        let mut step: usize = 0;
        while step < 600 {
            let r = lcg(&mut state);
            let op = r >> 3;
            let v = ((op >> 7) % 200) as i64 - 100;
            let room = |h: &FibHeap| h.used + 1 < h.key.len();
            if op % 3 == 0 {
                if room(&a) {
                    a = fib_insert(a, v);
                    ka.push(v);
                    check(&a, "rand insert a");
                }
            } else if op % 3 == 1 {
                if room(&b) {
                    b = fib_insert(b, v);
                    kb.push(v);
                    check(&b, "rand insert b");
                }
            } else if a.min != 0 {
                let (a2, k) = fib_extract_min(a);
                a = a2;
                check(&a, "rand extract a");
                let x = k.expect("non-empty");
                assert_eq!(x, *sorted(ka.clone()).first().unwrap(), "wrong minimum");
                let mut i = 0;
                while i < ka.len() {
                    if ka[i] == x {
                        ka.remove(i);
                        break;
                    }
                    i += 1;
                }
            }
            // occasionally meld b into a
            if step % 97 == 96 && a.used + b.used + 1 < cap {
                let mut m = live_keys(&a);
                let mut mb = live_keys(&b);
                a = fib_merge(a, b);
                b = fib_new(cap);
                check(&a, "rand merge");
                m.append(&mut mb);
                assert_eq!(sorted(live_keys(&a)), sorted(m));
                ka.append(&mut kb);
            }
            assert_eq!(sorted(live_keys(&a)), sorted(ka.clone()));
            if ka.is_empty() {
                assert_eq!(a.min, 0);
            } else {
                assert!(a.min != 0);
            }
            for &v in &reachable(&a) {
                let d = a.degree[v];
                if d > maxdeg {
                    maxdeg = d;
                }
            }
            step += 1;
        }
        // the run really exercises the linking invariant at several degrees
        assert!(maxdeg >= 4, "only reached degree {}", maxdeg);
    }
}
