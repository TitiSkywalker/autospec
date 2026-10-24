use ds_red_black_tree::{rbt_insert, rbt_delete, rbt_find, Tree, Color};

// -- structural invariant checker (translated alongside the tests) ----------

fn is_red_node(t: &Tree) -> bool {
    match t {
        Tree::Node(c, _, _, _) => match c {
            Color::Red => true,
            _ => false,
        },
        _ => false,
    }
}

/// No red node has a red child.
fn no_red_red(t: &Tree) -> bool {
    match t {
        Tree::Empty => true,
        Tree::DoubleBlackEmpty => false,
        Tree::Node(c, l, _, r) => {
            let self_red = match c {
                Color::Red => true,
                _ => false,
            };
            let bad = if self_red {
                if is_red_node(&**l) {
                    true
                } else {
                    is_red_node(&**r)
                }
            } else {
                false
            };
            if bad {
                false
            } else {
                no_red_red(&**l) && no_red_red(&**r)
            }
        }
    }
}

/// No transient double-black state may remain in a valid tree.
fn no_db(t: &Tree) -> bool {
    match t {
        Tree::Empty => true,
        Tree::DoubleBlackEmpty => false,
        Tree::Node(c, l, _, r) => {
            let is_db = match c {
                Color::DoubleBlack => true,
                _ => false,
            };
            if is_db {
                false
            } else {
                no_db(&**l) && no_db(&**r)
            }
        }
    }
}

/// Black height along the left spine (accumulator form).  For a balanced
/// tree this equals the black height of any root-to-leaf path.
fn bh_go(t: &Tree, acc: usize) -> usize {
    match t {
        Tree::Empty => acc,
        Tree::DoubleBlackEmpty => acc,
        Tree::Node(c, l, _, _) => {
            match c {
                Color::Black => bh_go(&**l, acc + 1),
                _ => bh_go(&**l, acc),
            }
        }
    }
}

fn black_height(t: &Tree) -> usize {
    bh_go(t, 0)
}

/// Every node's left and right subtrees have equal black height.
fn bh_balanced(t: &Tree) -> bool {
    match t {
        Tree::Empty => true,
        Tree::DoubleBlackEmpty => false,
        Tree::Node(_, l, _, r) => {
            let hl = black_height(&**l);
            let hr = black_height(&**r);
            if hl != hr {
                false
            } else {
                bh_balanced(&**l) && bh_balanced(&**r)
            }
        }
    }
}

/// In-order traversal, collecting keys into `acc`.
fn inorder(t: &Tree, acc: &mut Vec<i32>) {
    match t {
        Tree::Empty => {}
        Tree::DoubleBlackEmpty => {}
        Tree::Node(_, l, k, r) => {
            inorder(&**l, acc);
            acc.push(*k);
            inorder(&**r, acc);
        }
    }
}

/// Keys are strictly increasing (a set: no duplicates, no inversions).
fn is_sorted(keys: &Vec<i32>) -> bool {
    let mut i: usize = 1;
    while i < keys.len() {
        if keys[i - 1] >= keys[i] {
            return false;
        }
        i += 1;
    }
    true
}

fn is_valid(t: &Tree) -> bool {
    let root_black = match t {
        Tree::Empty => true,
        Tree::Node(c, _, _, _) => match c {
            Color::Black => true,
            _ => false,
        },
        Tree::DoubleBlackEmpty => false,
    };
    let mut ks: Vec<i32> = Vec::new();
    inorder(t, &mut ks);
    root_black && no_red_red(t) && no_db(t) && bh_balanced(t) && is_sorted(&ks)
}

// -- tests ------------------------------------------------------------------

#[test]
fn test_insert_find() {
    let mut t = Tree::Empty;
    t = rbt_insert(t, 5);
    t = rbt_insert(t, 3);
    t = rbt_insert(t, 7);
    t = rbt_insert(t, 2);
    t = rbt_insert(t, 4);
    t = rbt_insert(t, 6);
    t = rbt_insert(t, 8);
    assert!(rbt_find(&t, 5));
    assert!(rbt_find(&t, 3));
    assert!(rbt_find(&t, 7));
    assert!(rbt_find(&t, 2));
    assert!(rbt_find(&t, 4));
    assert!(rbt_find(&t, 6));
    assert!(rbt_find(&t, 8));
    assert!(!rbt_find(&t, 1));
    assert!(!rbt_find(&t, 9));
    assert!(is_valid(&t));
}

#[test]
fn test_insert_duplicate() {
    let mut t = Tree::Empty;
    t = rbt_insert(t, 10);
    t = rbt_insert(t, 10);
    t = rbt_insert(t, 10);
    assert!(rbt_find(&t, 10));
    assert!(!rbt_find(&t, 9));
    assert!(is_valid(&t));
}

#[test]
fn test_find_empty() {
    let t = Tree::Empty;
    assert!(!rbt_find(&t, 42));
}

#[test]
fn test_insert_many_valid() {
    let mut t = Tree::Empty;
    let mut i: i32 = -20;
    while i <= 20 {
        t = rbt_insert(t, i);
        i += 1;
    }
    assert!(is_valid(&t));
    let mut i: i32 = -20;
    while i <= 20 {
        assert!(rbt_find(&t, i));
        i += 1;
    }
    assert!(!rbt_find(&t, 100));
    assert!(!rbt_find(&t, -100));
}

#[test]
fn test_delete_even() {
    let mut t = Tree::Empty;
    let mut i: i32 = 0;
    while i < 20 {
        t = rbt_insert(t, i);
        i += 1;
    }
    let mut i: i32 = 0;
    while i < 20 {
        if i % 2 == 0 {
            t = rbt_delete(t, i);
        }
        i += 1;
    }
    assert!(is_valid(&t));
    let mut i: i32 = 0;
    while i < 20 {
        if i % 2 == 0 {
            assert!(!rbt_find(&t, i));
        } else {
            assert!(rbt_find(&t, i));
        }
        i += 1;
    }
}

#[test]
fn test_delete_all() {
    let mut t = Tree::Empty;
    let mut i: i32 = 0;
    while i < 10 {
        t = rbt_insert(t, i);
        i += 1;
    }
    let mut i: i32 = 0;
    while i < 10 {
        t = rbt_delete(t, i);
        i += 1;
    }
    assert!(is_valid(&t));
    let mut i: i32 = 0;
    while i < 10 {
        assert!(!rbt_find(&t, i));
        i += 1;
    }
}

#[test]
fn test_delete_absent() {
    let mut t = Tree::Empty;
    t = rbt_insert(t, 5);
    t = rbt_insert(t, 3);
    t = rbt_insert(t, 7);
    t = rbt_delete(t, 99);
    assert!(rbt_find(&t, 5));
    assert!(rbt_find(&t, 3));
    assert!(rbt_find(&t, 7));
    assert!(!rbt_find(&t, 99));
    assert!(is_valid(&t));
}

#[test]
fn test_insert_delete_interleaved() {
    let mut t = Tree::Empty;
    t = rbt_insert(t, 4);
    t = rbt_insert(t, 1);
    t = rbt_delete(t, 1);
    t = rbt_insert(t, 2);
    t = rbt_insert(t, 9);
    t = rbt_delete(t, 4);
    t = rbt_insert(t, 3);
    t = rbt_insert(t, 1);
    t = rbt_delete(t, 9);
    assert!(rbt_find(&t, 1));
    assert!(rbt_find(&t, 2));
    assert!(rbt_find(&t, 3));
    assert!(!rbt_find(&t, 4));
    assert!(!rbt_find(&t, 9));
    assert!(is_valid(&t));
}
