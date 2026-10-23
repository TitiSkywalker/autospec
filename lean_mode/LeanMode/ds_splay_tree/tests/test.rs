use ds_splay_tree::{splay, splay_contains, splay_find, splay_insert, Node};

/// In-order key list, collected into `acc`.
fn t_collect(t: &Option<Box<Node>>, acc: &mut Vec<i32>) {
    match t {
        None => {}
        Some(n) => {
            t_collect(&n.left, acc);
            acc.push(n.key);
            t_collect(&n.right, acc);
        }
    }
}

/// Number of keys in the tree.
fn t_count(t: &Option<Box<Node>>) -> usize {
    let mut v: Vec<i32> = Vec::new();
    t_collect(t, &mut v);
    v.len()
}

/// The key at the root (`-1` when empty).
fn t_top(t: &Option<Box<Node>>) -> i32 {
    match t {
        None => -1,
        Some(n) => n.key,
    }
}

/// The key at position `i` of the in-order list (`-1` when out of range).
fn t_at(t: &Option<Box<Node>>, i: usize) -> i32 {
    let mut v: Vec<i32> = Vec::new();
    t_collect(t, &mut v);
    if i < v.len() {
        v[i]
    } else {
        -1
    }
}

/// Build a tree by inserting the keys `a[start .. end)` of `a`.
fn t_build(a: &[i32], start: usize, end: usize) -> Option<Box<Node>> {
    let mut t: Option<Box<Node>> = None;
    let mut i = start;
    while i < end {
        t = splay_insert(t, a[i]);
        i += 1;
    }
    t
}

#[test]
fn test_insert_splays_to_root() {
    let a = vec![50, 30, 70, 20, 40];
    let t = t_build(&a, 0, 5);
    assert_eq!(t_top(&t), 40);
    assert_eq!(t_count(&t), 5);
    assert_eq!(t_at(&t, 0), 20);
    assert_eq!(t_at(&t, 4), 70);
}

#[test]
fn test_splay_moves_key_to_root() {
    let a = vec![10, 5, 15, 3, 7, 12, 20];
    let t = t_build(&a, 0, 7);
    let t2 = splay(t, 3);
    assert_eq!(t_top(&t2), 3);
    assert_eq!(t_count(&t2), 7);
    assert_eq!(t_at(&t2, 0), 3);
    assert_eq!(t_at(&t2, 6), 20);
}

#[test]
fn test_splay_contains() {
    let a = vec![8, 4, 12, 2, 6];
    let t = t_build(&a, 0, 5);
    assert!(splay_contains(&t, 6));
    assert!(splay_contains(&t, 8));
    assert!(!splay_contains(&t, 5));
    assert!(!splay_contains(&t, 100));
}

#[test]
fn test_splay_find_present_and_absent() {
    let a = vec![9, 4, 14, 2, 7, 11, 18];
    let t = t_build(&a, 0, 7);
    let (t2, found) = splay_find(t, 7);
    assert!(found);
    assert_eq!(t_top(&t2), 7);
    assert_eq!(t_count(&t2), 7);
    let (t3, missing) = splay_find(t2, 5);
    assert!(!missing);
    assert_eq!(t_top(&t3), 7);
    assert_eq!(t_count(&t3), 7);
}

#[test]
fn test_duplicate_insert() {
    let mut t: Option<Box<Node>> = None;
    t = splay_insert(t, 7);
    t = splay_insert(t, 7);
    t = splay_insert(t, 3);
    assert_eq!(t_count(&t), 2);
    assert_eq!(t_top(&t), 3);
}

#[test]
fn test_increasing_and_decreasing_order() {
    // Sorted insertion is the worst case for a plain BST; splay keeps it
    // reorganised but the key set must stay intact.
    let mut t: Option<Box<Node>> = None;
    let mut i: i32 = 1;
    while i <= 16 {
        t = splay_insert(t, i);
        i += 1;
    }
    assert_eq!(t_count(&t), 16);
    assert_eq!(t_top(&t), 16);
    // splay the smallest key; it must become the root
    let t2 = splay(t, 1);
    assert_eq!(t_top(&t2), 1);
    assert_eq!(t_count(&t2), 16);
    assert_eq!(t_at(&t2, 0), 1);
    assert_eq!(t_at(&t2, 15), 16);
}
