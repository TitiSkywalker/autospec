//! Binary Search Tree — insert and find operations.
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