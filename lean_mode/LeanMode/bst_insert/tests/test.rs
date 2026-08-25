use bst_insert::{bst_insert, bst_find, Node};

/// Helper: build a tree from a slice of values inserted in order.
fn build_tree(values: &[i32]) -> Option<Box<Node>> {
    let mut root: Option<Box<Node>> = None;
    let mut i: usize = 0;
    while i < values.len() {
        root = bst_insert(root, values[i]);
        i += 1;
    }
    root
}

/// Helper: collect values from the tree into a Vec via in-order traversal.
fn inorder_collect(node: &Option<Box<Node>>, acc: &mut Vec<i32>) {
    match node {
        None => {}
        Some(current) => {
            inorder_collect(&current.left, acc);
            acc.push(current.value);
            inorder_collect(&current.right, acc);
        }
    }
}

#[test]
fn test_insert_empty() {
    let root: Option<Box<Node>> = None;
    let root = bst_insert(root, 42);
    assert!(bst_find(&root, 42));
}

#[test]
fn test_insert_single_find_present() {
    let root: Option<Box<Node>> = None;
    let root = bst_insert(root, 10);
    assert!(bst_find(&root, 10));
}

#[test]
fn test_insert_single_find_absent() {
    let root: Option<Box<Node>> = None;
    let root = bst_insert(root, 10);
    assert!(!bst_find(&root, 5));
    assert!(!bst_find(&root, 15));
}

#[test]
fn test_insert_multiple() {
    let root = build_tree(&[5, 3, 7, 2, 4, 6, 8]);
    assert!(bst_find(&root, 5));
    assert!(bst_find(&root, 3));
    assert!(bst_find(&root, 7));
    assert!(bst_find(&root, 2));
    assert!(bst_find(&root, 4));
    assert!(bst_find(&root, 6));
    assert!(bst_find(&root, 8));
    assert!(!bst_find(&root, 1));
    assert!(!bst_find(&root, 9));
}

#[test]
fn test_insert_duplicate() {
    let root: Option<Box<Node>> = None;
    let root = bst_insert(root, 10);
    let root = bst_insert(root, 10);
    assert!(bst_find(&root, 10));
}

#[test]
fn test_inorder_sorted() {
    let root = build_tree(&[5, 3, 7, 2, 4, 6, 8]);
    let mut collected: Vec<i32> = Vec::new();
    inorder_collect(&root, &mut collected);
    let expected = vec![2, 3, 4, 5, 6, 7, 8];
    let mut i: usize = 0;
    assert_eq!(collected.len(), expected.len());
    while i < collected.len() {
        assert_eq!(collected[i], expected[i]);
        i += 1;
    }
}

#[test]
fn test_insert_negative_values() {
    let root = build_tree(&[0, -5, 5, -10, -3, 3, 10]);
    assert!(bst_find(&root, -10));
    assert!(bst_find(&root, -5));
    assert!(bst_find(&root, -3));
    assert!(bst_find(&root, 0));
    assert!(bst_find(&root, 3));
    assert!(bst_find(&root, 5));
    assert!(bst_find(&root, 10));
}

#[test]
fn test_find_on_empty() {
    let root: Option<Box<Node>> = None;
    assert!(!bst_find(&root, 42));
}