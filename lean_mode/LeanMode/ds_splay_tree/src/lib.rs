//! Splay tree — a self-adjusting binary search tree.
//!
//! Every access (`splay`) restructures the tree by rotations so that the
//! accessed key becomes the root.  Unlike the local rebalancing of an AVL
//! tree or a red-black tree, the balance here is *amortized*: a single splay
//! can leave the tree badly unbalanced, but a sequence of accesses is cheap.
//!
//! `splay` is the standard bottom-up recursive formulation: it descends to
//! the key by recursion, splays the deeper subtree first, and then performs
//! the zig / zig-zig / zig-zag rotations on the way back up.  `Box` is erased
//! by Aeneas, yielding a pure inductive `Node` in Lean.

/// A splay-tree node: BST key plus left/right children.
pub struct Node {
    pub key: i32,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

/// Build a node.
fn mk(key: i32, left: Option<Box<Node>>, right: Option<Box<Node>>) -> Node {
    Node { key, left, right }
}

/// Right rotation around `t` (the left child becomes the new subtree root).
/// A no-op when `t` has no left child.
fn rot_right(t: Node) -> Node {
    match t.left {
        None => t,
        Some(l_box) => {
            let l = *l_box;
            let new_right = mk(t.key, l.right, t.right);
            mk(l.key, l.left, Some(Box::new(new_right)))
        }
    }
}

/// Left rotation around `t` (the right child becomes the new subtree root).
/// A no-op when `t` has no right child.
fn rot_left(t: Node) -> Node {
    match t.right {
        None => t,
        Some(r_box) => {
            let r = *r_box;
            let new_left = mk(t.key, t.left, r.left);
            mk(r.key, Some(Box::new(new_left)), r.right)
        }
    }
}

/// Splay `key` to the root of the tree.
///
/// The recursion descends along the search path; on the way back up each
/// level applies the rotation dictated by the position of `key` in the
/// grandchild subtree: zig-zig (two rotations on the same side), zig-zag
/// (rotations on opposite sides) or zig (a single rotation).
///
/// When `key` is present the returned tree is a BST with the same key set
/// whose root is the node holding `key`.  When `key` is absent the tree is
/// still restructured (the last node on the search path is splayed), which is
/// the usual "splay the last accessed node" convention.
pub fn splay(t: Option<Box<Node>>, key: i32) -> Option<Box<Node>> {
    match t {
        None => None,
        Some(n_box) => {
            let n = *n_box;
            if key < n.key {
                match n.left {
                    None => Some(Box::new(n)),
                    Some(l_box) => {
                        let l = *l_box;
                        if key < l.key {
                            // zig-zig: splay into l.left, then rotate right twice
                            let ll = splay(l.left, key);
                            let l2 = mk(l.key, ll, l.right);
                            let n2 = mk(n.key, Some(Box::new(l2)), n.right);
                            let n3 = rot_right(n2);
                            Some(Box::new(rot_right(n3)))
                        } else if key > l.key {
                            // zig-zag: splay into l.right, then rotate left, right
                            let lr = splay(l.right, key);
                            let l2 = mk(l.key, l.left, lr);
                            let l3 = rot_left(l2);
                            let n2 = mk(n.key, Some(Box::new(l3)), n.right);
                            Some(Box::new(rot_right(n2)))
                        } else {
                            // zig: l holds the key
                            let n2 = mk(n.key, Some(Box::new(l)), n.right);
                            Some(Box::new(rot_right(n2)))
                        }
                    }
                }
            } else if key > n.key {
                match n.right {
                    None => Some(Box::new(n)),
                    Some(r_box) => {
                        let r = *r_box;
                        if key > r.key {
                            // zig-zig: splay into r.right, then rotate left twice
                            let rr = splay(r.right, key);
                            let r2 = mk(r.key, r.left, rr);
                            let n2 = mk(n.key, n.left, Some(Box::new(r2)));
                            let n3 = rot_left(n2);
                            Some(Box::new(rot_left(n3)))
                        } else if key < r.key {
                            // zig-zag: splay into r.left, then rotate right, left
                            let rl = splay(r.left, key);
                            let r2 = mk(r.key, rl, r.right);
                            let r3 = rot_right(r2);
                            let n2 = mk(n.key, n.left, Some(Box::new(r3)));
                            Some(Box::new(rot_left(n2)))
                        } else {
                            // zig: r holds the key
                            let n2 = mk(n.key, n.left, Some(Box::new(r)));
                            Some(Box::new(rot_left(n2)))
                        }
                    }
                }
            } else {
                // the root itself holds the key
                Some(Box::new(n))
            }
        }
    }
}

/// Plain BST insert (no rebalancing).
fn bst_insert(t: Option<Box<Node>>, key: i32) -> Option<Box<Node>> {
    match t {
        None => Some(Box::new(mk(key, None, None))),
        Some(n_box) => {
            let n = *n_box;
            if key < n.key {
                let new_left = bst_insert(n.left, key);
                Some(Box::new(mk(n.key, new_left, n.right)))
            } else if key > n.key {
                let new_right = bst_insert(n.right, key);
                Some(Box::new(mk(n.key, n.left, new_right)))
            } else {
                Some(Box::new(n))
            }
        }
    }
}

/// Insert `key` and splay it to the root.  Inserting an existing key leaves
/// the key set unchanged (the tree is a set) but still splays that key.
pub fn splay_insert(t: Option<Box<Node>>, key: i32) -> Option<Box<Node>> {
    let t2 = bst_insert(t, key);
    splay(t2, key)
}

/// Membership test, without restructuring.
pub fn splay_contains(t: &Option<Box<Node>>, key: i32) -> bool {
    match t {
        None => false,
        Some(n) => {
            if key < n.key {
                splay_contains(&n.left, key)
            } else if key > n.key {
                splay_contains(&n.right, key)
            } else {
                true
            }
        }
    }
}

/// Search for `key`, splaying it to the root when found.  Returns the
/// (possibly restructured) tree together with whether `key` was present.
pub fn splay_find(t: Option<Box<Node>>, key: i32) -> (Option<Box<Node>>, bool) {
    let found = splay_contains(&t, key);
    if found {
        let t2 = splay(t, key);
        (t2, true)
    } else {
        (t, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t_inorder(t: &Option<Box<Node>>, acc: &mut Vec<i32>) {
        match t {
            None => {}
            Some(n) => {
                t_inorder(&n.left, acc);
                acc.push(n.key);
                t_inorder(&n.right, acc);
            }
        }
    }

    fn t_keys(t: &Option<Box<Node>>) -> Vec<i32> {
        let mut v: Vec<i32> = Vec::new();
        t_inorder(t, &mut v);
        v
    }

    /// Keys strictly increasing (the tree stores a set, no duplicates)?
    fn t_strict_sorted(v: &Vec<i32>) -> bool {
        let mut i: usize = 1;
        while i < v.len() {
            if v[i - 1] >= v[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    fn t_is_bst(t: &Option<Box<Node>>) -> bool {
        let v = t_keys(t);
        t_strict_sorted(&v)
    }

    fn t_size(t: &Option<Box<Node>>) -> usize {
        let v = t_keys(t);
        v.len()
    }

    fn t_root(t: &Option<Box<Node>>) -> i32 {
        match t {
            None => -1,
            Some(n) => n.key,
        }
    }

    fn t_lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    /// Count of distinct values of `v` (after sorting a copy).
    fn t_distinct(a: &Vec<i32>) -> usize {
        let mut v = a.clone();
        let mut i: usize = 0;
        while i < v.len() {
            let mut j: usize = 0;
            while j + 1 < v.len() {
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
        let mut c: usize = 0;
        let mut i: usize = 0;
        while i < v.len() {
            if i == 0 {
                c += 1;
            } else if v[i - 1] != v[i] {
                c += 1;
            }
            i += 1;
        }
        c
    }

    fn t_present(a: &Vec<i32>, k: i32) -> bool {
        let mut i: usize = 0;
        let mut found = false;
        while i < a.len() {
            if a[i] == k {
                found = true;
            }
            i += 1;
        }
        found
    }

    #[test]
    fn insert_keeps_set_and_splays_root() {
        let mut t: Option<Box<Node>> = None;
        let keys = vec![50, 30, 70, 20, 40, 60, 80, 10];
        let mut i: usize = 0;
        while i < keys.len() {
            let k = keys[i];
            t = splay_insert(t, k);
            assert!(t_is_bst(&t));
            assert_eq!(t_root(&t), k);
            assert_eq!(t_size(&t), i + 1);
            i += 1;
        }
    }

    #[test]
    fn splay_moves_present_key_to_root() {
        let mut t: Option<Box<Node>> = None;
        let mut i: i32 = 1;
        while i <= 20 {
            t = splay_insert(t, i * 3);
            i += 1;
        }
        let mut j: i32 = 1;
        while j <= 20 {
            let t2 = splay(t, j * 3);
            assert_eq!(t_root(&t2), j * 3);
            assert!(t_is_bst(&t2));
            assert_eq!(t_size(&t2), 20);
            t = t2;
            j += 1;
        }
    }

    #[test]
    fn splay_find_reports_membership() {
        let mut t: Option<Box<Node>> = None;
        t = splay_insert(t, 10);
        t = splay_insert(t, 5);
        t = splay_insert(t, 15);
        let (t2, b) = splay_find(t, 5);
        assert!(b);
        assert_eq!(t_root(&t2), 5);
        let (t3, c) = splay_find(t2, 99);
        assert!(!c);
        assert_eq!(t_root(&t3), 5);
        assert_eq!(t_size(&t3), 3);
    }

    #[test]
    fn duplicate_insert_keeps_size() {
        let mut t: Option<Box<Node>> = None;
        t = splay_insert(t, 7);
        t = splay_insert(t, 7);
        t = splay_insert(t, 3);
        t = splay_insert(t, 7);
        assert_eq!(t_size(&t), 2);
        assert_eq!(t_root(&t), 7);
        assert!(t_is_bst(&t));
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 20240607;
        let mut t: Option<Box<Node>> = None;
        let mut refs: Vec<i32> = Vec::new();
        let mut step: usize = 0;
        while step < 400 {
            let r = t_lcg(&mut state);
            let k = ((r >> 20) % 100) as i32;
            let op = (r >> 3) % 3;
            if op < 2 {
                t = splay_insert(t, k);
                refs.push(k);
                assert_eq!(t_root(&t), k);
            } else {
                let (t2, b) = splay_find(t, k);
                t = t2;
                assert!(b == t_present(&refs, k));
                if b {
                    assert_eq!(t_root(&t), k);
                }
            }
            assert!(t_is_bst(&t));
            assert_eq!(t_size(&t), t_distinct(&refs));
            step += 1;
        }
    }
}
