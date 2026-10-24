//! Treap (a binary search tree whose priorities form a max-heap).
//!
//! A persistent treap keyed by `i32` with an explicit `i32` priority.  The
//! tree is a BST on `key` and a max-heap on `priority`; `treap_merge` and
//! `treap_split` are the two structural primitives, and insert/delete are
//! expressed in terms of them.  `Box` is erased by Aeneas, yielding a pure
//! inductive `Node` in Lean.

/// A treap node: BST key, heap priority, and left/right children.
pub struct Node {
    pub key: i32,
    pub priority: i32,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

/// Merge two treaps, assuming every key in `a` is smaller than every key in
/// `b`.  Returns a treap containing the union of both key sets.
pub fn treap_merge(a: Option<Box<Node>>, b: Option<Box<Node>>) -> Option<Box<Node>> {
    match a {
        None => b,
        Some(an) => match b {
            None => Some(an),
            Some(bn) => {
                if an.priority > bn.priority {
                    let new_right = treap_merge(an.right, Some(bn));
                    Some(Box::new(Node {
                        key: an.key,
                        priority: an.priority,
                        left: an.left,
                        right: new_right,
                    }))
                } else {
                    let new_left = treap_merge(Some(an), bn.left);
                    Some(Box::new(Node {
                        key: bn.key,
                        priority: bn.priority,
                        left: new_left,
                        right: bn.right,
                    }))
                }
            }
        },
    }
}

/// Split a treap at `key`: the left result holds every key `< key` and the
/// right result every key `>= key`.  Both results are valid treaps.
pub fn treap_split(t: Option<Box<Node>>, key: i32) -> (Option<Box<Node>>, Option<Box<Node>>) {
    match t {
        None => (None, None),
        Some(n) => {
            if n.key < key {
                let (l, r) = treap_split(n.right, key);
                (Some(Box::new(Node {
                    key: n.key,
                    priority: n.priority,
                    left: n.left,
                    right: l,
                })), r)
            } else {
                let (l, r) = treap_split(n.left, key);
                (l, Some(Box::new(Node {
                    key: n.key,
                    priority: n.priority,
                    left: r,
                    right: n.right,
                })))
            }
        }
    }
}

/// Insert `(key, priority)` into the treap.  Inserting an existing key leaves
/// the tree unchanged.
pub fn treap_insert(t: Option<Box<Node>>, key: i32, priority: i32) -> Option<Box<Node>> {
    if treap_find(&t, key) {
        t
    } else {
        let (l, r) = treap_split(t, key);
        let node = Some(Box::new(Node { key, priority, left: None, right: None }));
        treap_merge(treap_merge(l, node), r)
    }
}

/// Delete `key` from the treap.  Deleting an absent key leaves the tree
/// unchanged.
pub fn treap_delete(t: Option<Box<Node>>, key: i32) -> Option<Box<Node>> {
    match t {
        None => None,
        Some(n) => {
            if key < n.key {
                let new_left = treap_delete(n.left, key);
                Some(Box::new(Node {
                    key: n.key,
                    priority: n.priority,
                    left: new_left,
                    right: n.right,
                }))
            } else if key > n.key {
                let new_right = treap_delete(n.right, key);
                Some(Box::new(Node {
                    key: n.key,
                    priority: n.priority,
                    left: n.left,
                    right: new_right,
                }))
            } else {
                treap_merge(n.left, n.right)
            }
        }
    }
}

/// Search for `key` in the treap.
pub fn treap_find(t: &Option<Box<Node>>, key: i32) -> bool {
    match t {
        None => false,
        Some(n) => {
            if key < n.key {
                treap_find(&n.left, key)
            } else if key > n.key {
                treap_find(&n.right, key)
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

    fn is_heap(t: &Option<Box<Node>>) -> bool {
        match t {
            None => true,
            Some(n) => {
                let l_ok = match &n.left {
                    None => true,
                    Some(l) => n.priority >= l.priority,
                };
                let r_ok = match &n.right {
                    None => true,
                    Some(r) => n.priority >= r.priority,
                };
                l_ok && r_ok && is_heap(&n.left) && is_heap(&n.right)
            }
        }
    }

    fn is_treap(t: &Option<Box<Node>>) -> bool {
        is_bst(t) && is_heap(t)
    }

    // -- reference sorted set of (key, priority), unique keys ----------------

    fn contains_key(v: &Vec<(i32, i32)>, k: i32) -> bool {
        let mut i: usize = 0;
        while i < v.len() {
            if v[i].0 == k {
                return true;
            }
            i += 1;
        }
        false
    }

    fn insert_sorted(v: &mut Vec<(i32, i32)>, k: i32, p: i32) {
        if contains_key(v, k) {
            return;
        }
        let mut i: usize = 0;
        while i < v.len() && v[i].0 < k {
            i += 1;
        }
        v.insert(i, (k, p));
    }

    fn remove_sorted(v: &mut Vec<(i32, i32)>, k: i32) {
        let mut i: usize = 0;
        while i < v.len() {
            if v[i].0 == k {
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
        t = treap_insert(t, 5, 30);
        t = treap_insert(t, 3, 20);
        t = treap_insert(t, 7, 40);
        t = treap_insert(t, 2, 10);
        t = treap_insert(t, 4, 25);
        t = treap_insert(t, 6, 35);
        t = treap_insert(t, 8, 50);
        assert!(treap_find(&t, 2));
        assert!(treap_find(&t, 5));
        assert!(treap_find(&t, 8));
        assert!(!treap_find(&t, 1));
        assert!(!treap_find(&t, 9));
        assert!(is_treap(&t));
    }

    #[test]
    fn insert_duplicate_unchanged() {
        let mut t: Option<Box<Node>> = None;
        t = treap_insert(t, 10, 100);
        t = treap_insert(t, 10, 200);
        t = treap_insert(t, 10, 300);
        assert!(treap_find(&t, 10));
        assert!(is_treap(&t));
    }

    #[test]
    fn merge_two_ranges() {
        let mut a: Option<Box<Node>> = None;
        let mut b: Option<Box<Node>> = None;
        a = treap_insert(a, 1, 60);
        a = treap_insert(a, 3, 20);
        a = treap_insert(a, 2, 40);
        b = treap_insert(b, 5, 30);
        b = treap_insert(b, 4, 70);
        b = treap_insert(b, 6, 10);
        let m = treap_merge(a, b);
        assert!(is_treap(&m));
        let mut i = 1;
        while i <= 6 {
            assert!(treap_find(&m, i));
            i += 1;
        }
    }

    #[test]
    fn split_roundtrip() {
        let mut t: Option<Box<Node>> = None;
        let mut i: i32 = 0;
        while i < 20 {
            t = treap_insert(t, i, (i * 7919) % 101);
            i += 1;
        }
        let (l, r) = treap_split(t, 10);
        assert!(is_treap(&l));
        assert!(is_treap(&r));
        let mut i: i32 = 0;
        while i < 10 {
            assert!(treap_find(&l, i));
            assert!(!treap_find(&r, i));
            i += 1;
        }
        let mut i: i32 = 10;
        while i < 20 {
            assert!(!treap_find(&l, i));
            assert!(treap_find(&r, i));
            i += 1;
        }
    }

    #[test]
    fn delete_basic() {
        let mut t: Option<Box<Node>> = None;
        let mut i: i32 = 0;
        while i < 30 {
            t = treap_insert(t, i, (i * 31 + 7) % 97);
            i += 1;
        }
        let mut i: i32 = 0;
        while i < 30 {
            if i % 3 == 0 {
                t = treap_delete(t, i);
            }
            i += 1;
        }
        assert!(is_treap(&t));
        let mut i: i32 = 0;
        while i < 30 {
            assert_eq!(treap_find(&t, i), i % 3 != 0);
            i += 1;
        }
    }

    #[test]
    fn delete_absent() {
        let mut t: Option<Box<Node>> = None;
        t = treap_insert(t, 5, 10);
        t = treap_insert(t, 3, 20);
        t = treap_delete(t, 99);
        assert!(treap_find(&t, 5));
        assert!(treap_find(&t, 3));
        assert!(!treap_find(&t, 99));
        assert!(is_treap(&t));
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 987654321;
        let mut t: Option<Box<Node>> = None;
        let mut reference: Vec<(i32, i32)> = Vec::new();
        let mut ops: usize = 0;
        while ops < 20000 {
            let r = lcg(&mut state);
            let key = ((r >> 8) % 60) as i32 - 30;
            let priority = ((r >> 20) % 100000) as i32;
            if (r & 1) == 0 {
                t = treap_insert(t, key, priority);
                insert_sorted(&mut reference, key, priority);
            } else {
                t = treap_delete(t, key);
                remove_sorted(&mut reference, key);
            }
            assert!(is_treap(&t));
            let mut probe = -30;
            while probe <= 30 {
                assert_eq!(treap_find(&t, probe), contains_key(&reference, probe));
                probe += 1;
            }
            ops += 1;
        }
    }
}
