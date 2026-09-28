//! Binary Search Tree — insert, find, and delete.
//!
//! A recursive BST where each node stores an `i32` value and optional
//! left/right children.  `Box` is erased by Aeneas during translation,
//! yielding a pure inductive `Node` type in Lean.

/// A node in a binary search tree.
pub struct Node {
    pub value: i32,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}

/// Insert `value` into the BST rooted at `node`.
///
/// Returns the new root.  If `value` already exists the tree is unchanged.
pub fn bst_insert(node: Option<Box<Node>>, value: i32) -> Option<Box<Node>> {
    match node {
        None => {
            Some(Box::new(Node {
                value,
                left: None,
                right: None,
            }))
        }
        Some(current) => {
            if value < current.value {
                let new_left = bst_insert(current.left, value);
                Some(Box::new(Node {
                    value: current.value,
                    left: new_left,
                    right: current.right,
                }))
            } else if value > current.value {
                let new_right = bst_insert(current.right, value);
                Some(Box::new(Node {
                    value: current.value,
                    left: current.left,
                    right: new_right,
                }))
            } else {
                Some(current)
            }
        }
    }
}

/// Search for `value` in the BST rooted at `node`.
///
/// Returns `true` if the value is present, `false` otherwise.
pub fn bst_find(node: &Option<Box<Node>>, value: i32) -> bool {
    match node {
        None => false,
        Some(current) => {
            if value < current.value {
                bst_find(&current.left, value)
            } else if value > current.value {
                bst_find(&current.right, value)
            } else {
                true
            }
        }
    }
}

/// Remove the smallest node from `node` and return its value together with the
/// remaining tree.
fn extract_min(node: Box<Node>) -> (i32, Option<Box<Node>>) {
    match node.left {
        None => (node.value, node.right),
        Some(left) => {
            let (min, new_left) = extract_min(left);
            (min, Some(Box::new(Node {
                value: node.value,
                left: new_left,
                right: node.right,
            })))
        }
    }
}

/// Delete `value` from the BST rooted at `node`.
///
/// Returns the new root.  If the node has two children, it is replaced by the
/// minimum of its right subtree (its inorder successor).
pub fn bst_delete(node: Option<Box<Node>>, value: i32) -> Option<Box<Node>> {
    match node {
        None => None,
        Some(current) => {
            if value < current.value {
                let new_left = bst_delete(current.left, value);
                Some(Box::new(Node {
                    value: current.value,
                    left: new_left,
                    right: current.right,
                }))
            } else if value > current.value {
                let new_right = bst_delete(current.right, value);
                Some(Box::new(Node {
                    value: current.value,
                    left: current.left,
                    right: new_right,
                }))
            } else {
                match current.left {
                    None => current.right,
                    Some(left) => {
                        match current.right {
                            None => Some(left),
                            Some(right) => {
                                let (min, new_right) = extract_min(right);
                                Some(Box::new(Node {
                                    value: min,
                                    left: Some(left),
                                    right: new_right,
                                }))
                            }
                        }
                    }
                }
            }
        }
    }
}
