//! AVL Tree — a self-balancing binary search tree.
//!
//! Each node stores a `u64` subtree height (0 for the empty tree).  Insert
//! and delete rebalance with single/double rotations so that the heights of
//! every node's two children differ by at most one.  `Box` is erased by
//! Aeneas, yielding a pure inductive `Node` in Lean.

/// An AVL node: BST key, subtree height, and left/right children.
pub struct Node {
    pub key: i32,
    pub height: u64,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

/// Height of a (sub)tree; `0` for the empty tree.
fn height(t: &Option<Box<Node>>) -> u64 {
    match t {
        None => 0,
        Some(n) => n.height,
    }
}

/// `max(a, b)` (hand-expanded).
fn max_u64(a: u64, b: u64) -> u64 {
    if a > b {
        a
    } else {
        b
    }
}

/// Is `a` at least `b + 2`?  (Equivalent to balance factor `>= 2` without
/// needing subtraction on unsigned heights.)
fn taller_by_two(a: u64, b: u64) -> bool {
    let bp1 = b + 1;
    a > bp1
}

/// Construct a node with the correct height.
fn mk(key: i32, left: Option<Box<Node>>, right: Option<Box<Node>>) -> Node {
    let lh = height(&left);
    let rh = height(&right);
    let h = max_u64(lh, rh) + 1;
    Node { key, height: h, left, right }
}

/// Right rotation around `y` (whose left child becomes the new root).
fn rotate_right(y: Node) -> Node {
    match y.left {
        None => y,
        Some(x_box) => {
            let x = *x_box;
            let new_y = mk(y.key, x.right, y.right);
            mk(x.key, x.left, Some(Box::new(new_y)))
        }
    }
}

/// Left rotation around `x` (whose right child becomes the new root).
fn rotate_left(x: Node) -> Node {
    match x.right {
        None => x,
        Some(y_box) => {
            let y = *y_box;
            let new_x = mk(x.key, x.left, y.left);
            mk(y.key, Some(Box::new(new_x)), y.right)
        }
    }
}

/// Restore the AVL balance condition after an insert/delete on a child.
fn rebalance(n: Node) -> Node {
    let lh = height(&n.left);
    let rh = height(&n.right);
    if taller_by_two(lh, rh) {
        // left-heavy: left child must exist
        match n.left {
            Some(l_box) => {
                let l = *l_box;
                let llh = height(&l.left);
                let lrh = height(&l.right);
                if lrh > llh {
                    // left-right: rotate left child left, then rotate right
                    let new_left = rotate_left(l);
                    rotate_right(mk(n.key, Some(Box::new(new_left)), n.right))
                } else {
                    // left-left: rotate right
                    rotate_right(mk(n.key, Some(Box::new(l)), n.right))
                }
            }
            None => mk(n.key, None, n.right),
        }
    } else if taller_by_two(rh, lh) {
        // right-heavy: right child must exist
        match n.right {
            Some(r_box) => {
                let r = *r_box;
                let rlh = height(&r.left);
                let rrh = height(&r.right);
                if rlh > rrh {
                    // right-left: rotate right child right, then rotate left
                    let new_right = rotate_right(r);
                    rotate_left(mk(n.key, n.left, Some(Box::new(new_right))))
                } else {
                    // right-right: rotate left
                    rotate_left(mk(n.key, n.left, Some(Box::new(r))))
                }
            }
            None => mk(n.key, n.left, None),
        }
    } else {
        n
    }
}

/// Insert `key` into the tree (a set: an existing key leaves it unchanged).
pub fn avl_insert(t: Option<Box<Node>>, key: i32) -> Option<Box<Node>> {
    match t {
        None => Some(Box::new(mk(key, None, None))),
        Some(n) => {
            if key < n.key {
                let new_left = avl_insert(n.left, key);
                Some(Box::new(rebalance(mk(n.key, new_left, n.right))))
            } else if key > n.key {
                let new_right = avl_insert(n.right, key);
                Some(Box::new(rebalance(mk(n.key, n.left, new_right))))
            } else {
                Some(n)
            }
        }
    }
}

/// Remove the smallest node, returning its key and the updated subtree.
fn extract_min(n: Node) -> (i32, Option<Box<Node>>) {
    match n.left {
        None => (n.key, n.right),
        Some(l) => {
            let (min, new_left) = extract_min(*l);
            (min, Some(Box::new(rebalance(mk(n.key, new_left, n.right)))))
        }
    }
}

/// Delete `key` from the tree (deleting an absent key is a no-op).
pub fn avl_delete(t: Option<Box<Node>>, key: i32) -> Option<Box<Node>> {
    match t {
        None => None,
        Some(n) => {
            if key < n.key {
                let new_left = avl_delete(n.left, key);
                Some(Box::new(rebalance(mk(n.key, new_left, n.right))))
            } else if key > n.key {
                let new_right = avl_delete(n.right, key);
                Some(Box::new(rebalance(mk(n.key, n.left, new_right))))
            } else {
                match n.left {
                    None => n.right,
                    Some(l) => match n.right {
                        None => Some(l),
                        Some(r) => {
                            let (min, new_right) = extract_min(*r);
                            Some(Box::new(rebalance(mk(min, Some(l), new_right))))
                        }
                    },
                }
            }
        }
    }
}

/// Search for `key` in the tree.
pub fn avl_find(t: &Option<Box<Node>>, key: i32) -> bool {
    match t {
        None => false,
        Some(n) => {
            if key < n.key {
                avl_find(&n.left, key)
            } else if key > n.key {
                avl_find(&n.right, key)
            } else {
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- invariant checker (native-only; not translated) --------------------

    /// Recursively computed height (ignores the stored field, to also catch
    /// stale-height bugs).
    fn c_height(t: &Option<Box<Node>>) -> i32 {
        match t {
            None => 0,
            Some(n) => {
                let lh = c_height(&n.left);
                let rh = c_height(&n.right);
                let m = if lh > rh { lh } else { rh };
                1 + m
            }
        }
    }

    fn inorder(t: &Option<Box<Node>>, acc: &mut Vec<i32>) {
        match t {
            None => {}
            Some(n) => {
                inorder(&n.left, acc);
                acc.push(n.key);
                inorder(&n.right, acc);
            }
        }
    }

    fn is_bst(t: &Option<Box<Node>>) -> bool {
        let mut ks: Vec<i32> = Vec::new();
        inorder(t, &mut ks);
        let mut i: usize = 1;
        while i < ks.len() {
            if ks[i - 1] >= ks[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    fn is_avl(t: &Option<Box<Node>>) -> bool {
        match t {
            None => true,
            Some(n) => {
                let lh = c_height(&n.left);
                let rh = c_height(&n.right);
                let d = lh - rh;
                let balanced = if d > 1 {
                    false
                } else if d < -1 {
                    false
                } else {
                    true
                };
                let m = if lh > rh { lh } else { rh };
                let h_ok = (n.height as i32) == 1 + m;
                balanced && h_ok && is_avl(&n.left) && is_avl(&n.right)
            }
        }
    }

    fn is_valid(t: &Option<Box<Node>>) -> bool {
        is_bst(t) && is_avl(t)
    }

    // -- reference sorted set (Vec<i32>, unique keys) ------------------------

    fn contains(v: &Vec<i32>, k: i32) -> bool {
        let mut i: usize = 0;
        while i < v.len() {
            if v[i] == k {
                return true;
            }
            i += 1;
        }
        false
    }

    fn insert_sorted(v: &mut Vec<i32>, k: i32) {
        if contains(v, k) {
            return;
        }
        let mut i: usize = 0;
        while i < v.len() && v[i] < k {
            i += 1;
        }
        v.insert(i, k);
    }

    fn remove_sorted(v: &mut Vec<i32>, k: i32) {
        let mut i: usize = 0;
        while i < v.len() {
            if v[i] == k {
                v.remove(i);
                return;
            }
            i += 1;
        }
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *state
    }

    // -- tests -------------------------------------------------------------

    #[test]
    fn insert_find_basic() {
        let mut t: Option<Box<Node>> = None;
        t = avl_insert(t, 5);
        t = avl_insert(t, 3);
        t = avl_insert(t, 7);
        t = avl_insert(t, 2);
        t = avl_insert(t, 4);
        t = avl_insert(t, 6);
        t = avl_insert(t, 8);
        assert!(avl_find(&t, 2));
        assert!(avl_find(&t, 5));
        assert!(avl_find(&t, 8));
        assert!(!avl_find(&t, 1));
        assert!(!avl_find(&t, 9));
        assert!(is_valid(&t));
    }

    #[test]
    fn insert_duplicate_unchanged() {
        let mut t: Option<Box<Node>> = None;
        t = avl_insert(t, 10);
        t = avl_insert(t, 10);
        t = avl_insert(t, 10);
        assert!(avl_find(&t, 10));
        assert!(is_valid(&t));
    }

    #[test]
    fn insert_sorted_sequence_balanced() {
        // inserting keys in sorted order is the worst case for a plain BST;
        // the AVL rotations must keep the height logarithmic.
        let mut t: Option<Box<Node>> = None;
        let mut i: i32 = 0;
        while i < 100 {
            t = avl_insert(t, i);
            i += 1;
        }
        assert!(is_valid(&t));
        // height of 100 nodes must be far below 100 (log2(101) ≈ 7).
        let h = c_height(&t);
        assert!(h < 20);
    }

    #[test]
    fn delete_basic() {
        let mut t: Option<Box<Node>> = None;
        let mut i: i32 = 0;
        while i < 40 {
            t = avl_insert(t, i);
            i += 1;
        }
        let mut i: i32 = 0;
        while i < 40 {
            if i % 3 == 0 {
                t = avl_delete(t, i);
            }
            i += 1;
        }
        assert!(is_valid(&t));
        let mut i: i32 = 0;
        while i < 40 {
            assert_eq!(avl_find(&t, i), i % 3 != 0);
            i += 1;
        }
    }

    #[test]
    fn delete_absent() {
        let mut t: Option<Box<Node>> = None;
        t = avl_insert(t, 5);
        t = avl_insert(t, 3);
        t = avl_delete(t, 99);
        assert!(avl_find(&t, 5));
        assert!(avl_find(&t, 3));
        assert!(!avl_find(&t, 99));
        assert!(is_valid(&t));
    }

    #[test]
    fn delete_to_empty() {
        let mut t: Option<Box<Node>> = None;
        t = avl_insert(t, 7);
        t = avl_delete(t, 7);
        assert!(!avl_find(&t, 7));
        assert!(is_valid(&t));
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 123456789;
        let mut t: Option<Box<Node>> = None;
        let mut reference: Vec<i32> = Vec::new();
        let mut ops: usize = 0;
        while ops < 20000 {
            let r = lcg(&mut state);
            let key = ((r >> 8) % 80) as i32 - 40;
            if (r & 1) == 0 {
                t = avl_insert(t, key);
                insert_sorted(&mut reference, key);
            } else {
                t = avl_delete(t, key);
                remove_sorted(&mut reference, key);
            }
            assert!(is_valid(&t));
            let mut probe = -40;
            while probe <= 40 {
                assert_eq!(avl_find(&t, probe), contains(&reference, probe));
                probe += 1;
            }
            ops += 1;
        }
    }
}
