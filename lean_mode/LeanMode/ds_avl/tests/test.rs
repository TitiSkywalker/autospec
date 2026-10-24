use ds_avl::{avl_insert, avl_delete, avl_find, Node};

// -- structural invariant checker (translated alongside the tests) ----------

fn subtree_height(t: &Option<Box<Node>>) -> u64 {
    match t {
        None => 0,
        Some(n) => n.height,
    }
}

/// Is `a` at least `b + 2`?  (balance factor `>= 2` without subtraction.)
fn too_tall(a: u64, b: u64) -> bool {
    let bp1 = b + 1;
    a > bp1
}

/// AVL balance invariant: the two child heights differ by at most one.
fn is_avl(t: &Option<Box<Node>>) -> bool {
    match t {
        None => true,
        Some(n) => {
            let lh = subtree_height(&n.left);
            let rh = subtree_height(&n.right);
            if too_tall(lh, rh) {
                false
            } else if too_tall(rh, lh) {
                false
            } else {
                is_avl(&n.left) && is_avl(&n.right)
            }
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

fn is_valid(t: &Option<Box<Node>>) -> bool {
    is_bst(t) && is_avl(t)
}

// -- tests ------------------------------------------------------------------

#[test]
fn test_insert_find() {
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
fn test_insert_duplicate() {
    let mut t: Option<Box<Node>> = None;
    t = avl_insert(t, 10);
    t = avl_insert(t, 10);
    t = avl_insert(t, 10);
    assert!(avl_find(&t, 10));
    assert!(is_valid(&t));
}

#[test]
fn test_find_empty() {
    let t: Option<Box<Node>> = None;
    assert!(!avl_find(&t, 42));
}

#[test]
fn test_insert_sorted_sequence() {
    // sorted insertion is the worst case; rotations must keep it valid.
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 0;
    while i < 30 {
        t = avl_insert(t, i);
        i += 1;
    }
    assert!(is_valid(&t));
    let mut i: i32 = 0;
    while i < 30 {
        assert!(avl_find(&t, i));
        i += 1;
    }
}

#[test]
fn test_delete() {
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 0;
    while i < 20 {
        t = avl_insert(t, i);
        i += 1;
    }
    let mut i: i32 = 0;
    while i < 20 {
        if i % 2 == 0 {
            t = avl_delete(t, i);
        }
        i += 1;
    }
    assert!(is_valid(&t));
    let mut i: i32 = 0;
    while i < 20 {
        if i % 2 == 0 {
            assert!(!avl_find(&t, i));
        } else {
            assert!(avl_find(&t, i));
        }
        i += 1;
    }
}

#[test]
fn test_delete_absent() {
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
fn test_delete_all() {
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 0;
    while i < 10 {
        t = avl_insert(t, i);
        i += 1;
    }
    let mut i: i32 = 0;
    while i < 10 {
        t = avl_delete(t, i);
        i += 1;
    }
    assert!(is_valid(&t));
    let mut i: i32 = 0;
    while i < 10 {
        assert!(!avl_find(&t, i));
        i += 1;
    }
}
