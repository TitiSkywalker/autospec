use ds_wblt::{wblt_delete, wblt_find, wblt_insert, wblt_kth, wblt_rank, Node};

// -- structural invariant checker (translated alongside the tests) ----------

/// Stored subtree size (`0` for the empty tree).
fn sub_size(t: &Option<Box<Node>>) -> i32 {
    match t {
        None => 0,
        Some(n) => n.size,
    }
}

/// Is the lighter child too light?  `3·light < heavy`.
fn too_light_test(heavy: i32, light: i32) -> bool {
    let lhs = 3 * light;
    if lhs < heavy {
        true
    } else {
        false
    }
}

/// Weight-balance invariant: `3·min(lsz, rsz) ≥ max(lsz, rsz)` everywhere.
fn is_balanced(t: &Option<Box<Node>>) -> bool {
    match t {
        None => true,
        Some(n) => match &n.left {
            None => true,
            Some(_) => {
                let lsz = sub_size(&n.left);
                let rsz = sub_size(&n.right);
                if too_light_test(lsz, rsz) {
                    false
                } else if too_light_test(rsz, lsz) {
                    false
                } else {
                    is_balanced(&n.left) && is_balanced(&n.right)
                }
            }
        },
    }
}

fn inorder(t: &Option<Box<Node>>, acc: &mut Vec<i32>) {
    match t {
        None => {}
        Some(n) => match &n.left {
            None => acc.push(n.value),
            Some(_) => {
                inorder(&n.left, acc);
                inorder(&n.right, acc);
            }
        },
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
    is_bst(t) && is_balanced(t)
}

// -- tests ------------------------------------------------------------------

#[test]
fn test_insert_find() {
    let mut t: Option<Box<Node>> = None;
    t = wblt_insert(t, 5);
    t = wblt_insert(t, 3);
    t = wblt_insert(t, 7);
    t = wblt_insert(t, 2);
    t = wblt_insert(t, 4);
    t = wblt_insert(t, 6);
    t = wblt_insert(t, 8);
    assert!(wblt_find(&t, 2));
    assert!(wblt_find(&t, 5));
    assert!(wblt_find(&t, 8));
    assert!(!wblt_find(&t, 1));
    assert!(!wblt_find(&t, 9));
    assert!(is_valid(&t));
}

#[test]
fn test_insert_duplicate() {
    let mut t: Option<Box<Node>> = None;
    t = wblt_insert(t, 10);
    t = wblt_insert(t, 10);
    t = wblt_insert(t, 10);
    assert!(wblt_find(&t, 10));
    assert!(is_valid(&t));
}

#[test]
fn test_find_empty() {
    let t: Option<Box<Node>> = None;
    assert!(!wblt_find(&t, 42));
    assert_eq!(wblt_kth(&t, 1), -1);
    assert_eq!(wblt_rank(&t, 5), 0);
}

#[test]
fn test_insert_sorted_sequence() {
    // sorted insertion is the worst case; rebalancing must keep it valid.
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 0;
    while i < 30 {
        t = wblt_insert(t, i);
        i += 1;
    }
    assert!(is_valid(&t));
    let mut i: i32 = 0;
    while i < 30 {
        assert!(wblt_find(&t, i));
        i += 1;
    }
}

#[test]
fn test_delete() {
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 0;
    while i < 20 {
        t = wblt_insert(t, i);
        i += 1;
    }
    let (t2, found) = wblt_delete(t, 7);
    assert!(found);
    assert!(!wblt_find(&t2, 7));
    assert!(is_valid(&t2));

    let (t3, found2) = wblt_delete(t2, 0);
    assert!(found2);
    assert!(!wblt_find(&t3, 0));
    assert!(is_valid(&t3));

    // deleting an absent key is a no-op
    let (t4, found3) = wblt_delete(t3, 999);
    assert!(!found3);
    assert!(is_valid(&t4));
}

#[test]
fn test_kth_rank() {
    let mut t: Option<Box<Node>> = None;
    let keys = [5, 3, 8, 1, 4, 7, 9, 2, 6];
    let mut i = 0;
    while i < 9 {
        t = wblt_insert(t, keys[i]);
        i += 1;
    }
    assert!(is_valid(&t));
    assert_eq!(wblt_kth(&t, 1), 1);
    assert_eq!(wblt_kth(&t, 5), 5);
    assert_eq!(wblt_kth(&t, 9), 9);
    assert_eq!(wblt_kth(&t, 10), -1);
    assert_eq!(wblt_rank(&t, 1), 0);
    assert_eq!(wblt_rank(&t, 5), 4);
    assert_eq!(wblt_rank(&t, 10), 9);
}
