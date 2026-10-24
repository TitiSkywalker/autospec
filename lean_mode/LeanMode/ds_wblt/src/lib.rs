//! Persistent Weight-Balanced Leaf Tree (WBLT) — a self-balancing binary
//! search tree.
//!
//! WBLT is a *leafy* tree: keys live only in the leaves, and every internal
//! node has exactly two children and stores the maximum key of its subtree and
//! the number of leaves below it.  Balance is maintained by the *weight*
//! (subtree leaf count): every internal node keeps `3·min(lsz, rsz) ≥ max(lsz, rsz)`
//! (the α = 1/4 special case of the standard α-balance condition), rebalanced
//! with single/double rotations.
//!
//! All update operations are *persistent* (path-copying): `insert`/`delete`
//! return a new root and share every unchanged subtree with the old tree.
//! `Box` is erased by Aeneas, yielding a pure inductive `Node` in Lean.

/// A WBLT node.  A leaf has `left = None` (and `right = None`); an internal
/// node has two children and stores `value` = max key of the subtree and
/// `size` = number of leaves.
pub struct Node {
    pub value: i32,
    pub size: i32,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

/// Number of leaves in a (sub)tree; `0` for the empty tree.
fn size_of(t: &Option<Box<Node>>) -> i32 {
    match t {
        None => 0,
        Some(n) => n.size,
    }
}

/// Maximum key in a (sub)tree (rightmost leaf); `0` for the empty tree.
fn value_of(t: &Option<Box<Node>>) -> i32 {
    match t {
        None => 0,
        Some(n) => n.value,
    }
}

/// A fresh leaf holding `v`.
fn new_leaf(v: i32) -> Box<Node> {
    Box::new(Node {
        value: v,
        size: 1,
        left: None,
        right: None,
    })
}

/// Combine `left` and `right` into an internal node (recomputing size/value).
fn mk(left: Option<Box<Node>>, right: Option<Box<Node>>) -> Node {
    let lsz = size_of(&left);
    let rsz = size_of(&right);
    let v = value_of(&right);
    let sz = lsz + rsz;
    Node {
        value: v,
        size: sz,
        left,
        right,
    }
}

/// Is the lighter child too light?  i.e. `light < α·(heavy + light)` with
/// `α = 1/4`, rewritten to integer arithmetic as `3·light < heavy`.
fn too_light(heavy: i32, light: i32) -> bool {
    let lhs = 3 * light;
    if lhs < heavy {
        true
    } else {
        false
    }
}

/// Is a double rotation needed?  `inner > heavy/(2 − α)` with `α = 1/4`,
/// i.e. `7·inner > 4·heavy`.
fn need_double(inner: i32, heavy: i32) -> bool {
    let lhs = 7 * inner;
    let rhs = 4 * heavy;
    if lhs > rhs {
        true
    } else {
        false
    }
}

/// Rotate so that `node`'s `r`-child (0 = left, 1 = right) becomes the root.
/// A no-op when the child to promote is a leaf.
fn rotate(node: Node, r: bool) -> Node {
    if r {
        match node.right {
            None => node,
            Some(y_box) => {
                let y = *y_box;
                match y.left {
                    None => mk(node.left, Some(Box::new(y))),
                    Some(yl) => {
                        let new_x = mk(node.left, Some(yl));
                        mk(Some(Box::new(new_x)), y.right)
                    }
                }
            }
        }
    } else {
        match node.left {
            None => node,
            Some(y_box) => {
                let y = *y_box;
                match y.right {
                    None => mk(Some(Box::new(y)), node.right),
                    Some(yr) => {
                        let new_x = mk(Some(yr), node.right);
                        mk(y.left, Some(Box::new(new_x)))
                    }
                }
            }
        }
    }
}

/// Restore the weight-balance invariant after an update on a child.
fn balance(node: Node) -> Node {
    let lsz = size_of(&node.left);
    if lsz == 0 {
        node
    } else {
        let rsz = size_of(&node.right);
        let r_heavy = rsz > lsz;
        let heavy_sz = if r_heavy { rsz } else { lsz };
        let light_sz = if r_heavy { lsz } else { rsz };
        if too_light(heavy_sz, light_sz) {
            if r_heavy {
                match node.right {
                    Some(r_box) => {
                        let r = *r_box;
                        let inner = size_of(&r.left);
                        let r2 = if need_double(inner, r.size) {
                            rotate(r, false)
                        } else {
                            r
                        };
                        let new_node = mk(node.left, Some(Box::new(r2)));
                        rotate(new_node, true)
                    }
                    None => node,
                }
            } else {
                match node.left {
                    Some(l_box) => {
                        let l = *l_box;
                        let inner = size_of(&l.right);
                        let l2 = if need_double(inner, l.size) {
                            rotate(l, true)
                        } else {
                            l
                        };
                        let new_node = mk(Some(Box::new(l2)), node.right);
                        rotate(new_node, false)
                    }
                    None => node,
                }
            }
        } else {
            node
        }
    }
}

/// Insert `v` into the tree (a set: an existing key leaves it unchanged).
pub fn wblt_insert(t: Option<Box<Node>>, v: i32) -> Option<Box<Node>> {
    match t {
        None => Some(new_leaf(v)),
        Some(n) => match n.left {
            None => {
                // leaf
                let old = n.value;
                if v == old {
                    Some(n)
                } else if v < old {
                    Some(Box::new(mk(Some(new_leaf(v)), Some(new_leaf(old)))))
                } else {
                    Some(Box::new(mk(Some(new_leaf(old)), Some(new_leaf(v)))))
                }
            }
            Some(l) => {
                // internal
                if v > l.value {
                    let new_right = wblt_insert(n.right, v);
                    Some(Box::new(balance(mk(Some(l), new_right))))
                } else {
                    let new_left = wblt_insert(Some(l), v);
                    Some(Box::new(balance(mk(new_left, n.right))))
                }
            }
        },
    }
}

/// Delete `v` from the tree.  Returns the new root and whether `v` was found.
pub fn wblt_delete(t: Option<Box<Node>>, v: i32) -> (Option<Box<Node>>, bool) {
    match t {
        None => (None, false),
        Some(n) => match n.left {
            None => {
                // leaf
                if n.value == v {
                    (None, true)
                } else {
                    (Some(n), false)
                }
            }
            Some(l) => {
                // internal
                if v > l.value {
                    let (new_right, succ) = wblt_delete(n.right, v);
                    match new_right {
                        Some(nr) => (Some(Box::new(balance(mk(Some(l), Some(nr))))), succ),
                        None => (Some(l), succ),
                    }
                } else {
                    let (new_left, succ) = wblt_delete(Some(l), v);
                    match new_left {
                        Some(nl) => (Some(Box::new(balance(mk(Some(nl), n.right)))), succ),
                        None => (n.right, succ),
                    }
                }
            }
        },
    }
}

/// Membership test.
pub fn wblt_find(t: &Option<Box<Node>>, v: i32) -> bool {
    match t {
        None => false,
        Some(n) => {
            let lsz = size_of(&n.left);
            if lsz == 0 {
                if n.value == v {
                    true
                } else {
                    false
                }
            } else {
                let lv = value_of(&n.left);
                if v > lv {
                    wblt_find(&n.right, v)
                } else {
                    wblt_find(&n.left, v)
                }
            }
        }
    }
}

/// k-th smallest key (1-indexed); `-1` if out of range.
pub fn wblt_kth(t: &Option<Box<Node>>, k: i32) -> i32 {
    match t {
        None => -1,
        Some(n) => {
            let lsz = size_of(&n.left);
            if lsz == 0 {
                if k == 1 {
                    n.value
                } else {
                    -1
                }
            } else {
                if k > lsz {
                    let k2 = k - lsz;
                    wblt_kth(&n.right, k2)
                } else {
                    wblt_kth(&n.left, k)
                }
            }
        }
    }
}

/// Number of keys strictly smaller than `v`.
pub fn wblt_rank(t: &Option<Box<Node>>, v: i32) -> i32 {
    wblt_rank_acc(t, v, 0)
}

fn wblt_rank_acc(t: &Option<Box<Node>>, v: i32, acc: i32) -> i32 {
    match t {
        None => acc,
        Some(n) => {
            let lsz = size_of(&n.left);
            if lsz == 0 {
                if n.value < v {
                    acc + 1
                } else {
                    acc
                }
            } else {
                let lv = value_of(&n.left);
                if lv < v {
                    let new_acc = acc + lsz;
                    wblt_rank_acc(&n.right, v, new_acc)
                } else {
                    wblt_rank_acc(&n.left, v, acc)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In-order traversal into a Vec (native-only; not translated).
    fn to_vec(t: &Option<Box<Node>>, out: &mut Vec<i32>) {
        match t {
            None => {}
            Some(n) => match &n.left {
                None => out.push(n.value),
                Some(_) => {
                    to_vec(&n.left, out);
                    to_vec(&n.right, out);
                }
            },
        }
    }

    /// Brute-force size / value / balance checker (ignores stored fields where
    /// possible, to catch stale-field bugs).
    fn check(t: &Option<Box<Node>>) -> (i32, i32, bool) {
        // returns (size, max_value, ok)
        match t {
            None => (0, i32::MIN, true),
            Some(n) => match &n.left {
                None => {
                    let ok = n.size == 1;
                    (1, n.value, ok)
                }
                Some(_) => {
                    let (lsz, lmax, lok) = check(&n.left);
                    let (rsz, rmax, rok) = check(&n.right);
                    let ok = lok
                        && rok
                        && n.size == lsz + rsz
                        && n.value == rmax
                        && 3 * lsz >= rsz
                        && 3 * rsz >= lsz
                        && lmax <= rmax;
                    (n.size, rmax, ok)
                }
            },
        }
    }

    fn insert_all(keys: &[i32]) -> Option<Box<Node>> {
        let mut t = None;
        let mut i = 0;
        while i < keys.len() {
            t = wblt_insert(t, keys[i]);
            i += 1;
        }
        t
    }

    #[test]
    fn build_and_check_sorted() {
        let t = insert_all(&[5, 3, 8, 1, 4, 7, 9, 2, 6]);
        let mut v = Vec::new();
        to_vec(&t, &mut v);
        assert_eq!(v, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert!(check(&t).2);
    }

    #[test]
    fn random_insert_check() {
        let keys = [
            42, 17, 23, 8, 91, 5, 13, 77, 3, 29, 51, 64, 1, 99, 37, 70, 11, 25, 88, 6,
        ];
        let t = insert_all(&keys);
        assert!(check(&t).2);
        // membership
        assert!(wblt_find(&t, 13));
        assert!(wblt_find(&t, 99));
        assert!(!wblt_find(&t, 100));
        // kth / rank
        assert_eq!(wblt_kth(&t, 1), 1);
        assert_eq!(wblt_kth(&t, 20), 99);
        assert_eq!(wblt_kth(&t, 7), 13);
        assert_eq!(wblt_kth(&t, 8), 17);
        assert_eq!(wblt_rank(&t, 17), 7); // 1,3,5,6,8,11,13 are < 17
        assert_eq!(wblt_rank(&t, 1), 0);
    }

    #[test]
    fn random_insert_delete_check() {
        let keys = [
            5, 3, 8, 1, 4, 7, 9, 2, 6, 10, 12, 11, 15, 14, 13, 16, 20, 18, 19, 17,
        ];
        let mut t = None;
        let mut i = 0;
        while i < keys.len() {
            t = wblt_insert(t, keys[i]);
            assert!(check(&t).2);
            i += 1;
        }
        // delete half of them
        let dels = [3, 8, 14, 1, 20, 6, 11, 17, 5, 9];
        let mut d = 0;
        while d < dels.len() {
            let (t2, succ) = wblt_delete(t, dels[d]);
            assert!(succ);
            assert!(check(&t2).2);
            assert!(!wblt_find(&t2, dels[d]));
            t = t2;
            d += 1;
        }
        // deleting an absent key is a no-op
        let (t3, succ) = wblt_delete(t, 999);
        assert!(!succ);
        assert!(check(&t3).2);
    }
}
