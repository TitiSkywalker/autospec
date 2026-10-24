use ds_treap::{treap_insert, treap_delete, treap_find, treap_merge, treap_split, Node};

// -- structural invariant checker (translated alongside the tests) ----------

/// In-order traversal, collecting keys into `acc`.
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

/// Keys are strictly increasing (a set: no duplicates, no inversions).
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

/// `a >= b` (helper so the field accesses become call arguments, which the
/// translator tracks correctly).
fn ge(a: i32, b: i32) -> bool {
    if a >= b {
        true
    } else {
        false
    }
}

/// Max-heap invariant: every node's priority is at least its children's.
fn is_heap(t: &Option<Box<Node>>) -> bool {
    match t {
        None => true,
        Some(n) => {
            let l_ok = match &n.left {
                None => true,
                Some(l) => ge(n.priority, l.priority),
            };
            let r_ok = match &n.right {
                None => true,
                Some(r) => ge(n.priority, r.priority),
            };
            l_ok && r_ok && is_heap(&n.left) && is_heap(&n.right)
        }
    }
}

fn is_treap(t: &Option<Box<Node>>) -> bool {
    is_bst(t) && is_heap(t)
}

// -- tests ------------------------------------------------------------------

#[test]
fn test_insert_find() {
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
fn test_insert_duplicate() {
    let mut t: Option<Box<Node>> = None;
    t = treap_insert(t, 10, 100);
    t = treap_insert(t, 10, 200);
    t = treap_insert(t, 10, 300);
    assert!(treap_find(&t, 10));
    assert!(is_treap(&t));
}

#[test]
fn test_find_empty() {
    let t: Option<Box<Node>> = None;
    assert!(!treap_find(&t, 42));
}

#[test]
fn test_merge() {
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
    let mut i: i32 = 1;
    while i <= 6 {
        assert!(treap_find(&m, i));
        i += 1;
    }
}

#[test]
fn test_split() {
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
fn test_delete() {
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 0;
    while i < 20 {
        t = treap_insert(t, i, (i * 31 + 7) % 97);
        i += 1;
    }
    let mut i: i32 = 0;
    while i < 20 {
        if i % 2 == 0 {
            t = treap_delete(t, i);
        }
        i += 1;
    }
    assert!(is_treap(&t));
    let mut i: i32 = 0;
    while i < 20 {
        if i % 2 == 0 {
            assert!(!treap_find(&t, i));
        } else {
            assert!(treap_find(&t, i));
        }
        i += 1;
    }
}

#[test]
fn test_delete_absent() {
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
fn test_delete_all() {
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 0;
    while i < 10 {
        t = treap_insert(t, i, (i * 17 + 3) % 53);
        i += 1;
    }
    let mut i: i32 = 0;
    while i < 10 {
        t = treap_delete(t, i);
        i += 1;
    }
    assert!(is_treap(&t));
    let mut i: i32 = 0;
    while i < 10 {
        assert!(!treap_find(&t, i));
        i += 1;
    }
}
