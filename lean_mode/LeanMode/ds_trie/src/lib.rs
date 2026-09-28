//! Binary trie (radix-2 prefix tree) over bit-string keys.
//!
//! Each node has two children (`child0` / `child1`, for the next bit) and an
//! `is_end` flag marking the end of a stored key.  Keys are `&[usize]` where
//! each element is `0` or `1`.  `Box` is erased by Aeneas during translation.

/// A node in the trie.
pub struct TrieNode {
    pub child0: Option<Box<TrieNode>>,
    pub child1: Option<Box<TrieNode>>,
    pub is_end: bool,
}

/// Insert the suffix `key[idx..]` into the trie rooted at `node`; returns the new root.  The full key is inserted when `idx = 0`.
pub fn trie_insert(node: Option<Box<TrieNode>>, key: &[usize], idx: usize) -> Option<Box<TrieNode>> {
    if idx == key.len() {
        match node {
            None => {
                Some(Box::new(TrieNode {
                    child0: None,
                    child1: None,
                    is_end: true,
                }))
            }
            Some(current) => {
                Some(Box::new(TrieNode {
                    child0: current.child0,
                    child1: current.child1,
                    is_end: true,
                }))
            }
        }
    } else {
        let bit = key[idx];
        let (child0, child1, is_end) = match node {
            None => (None, None, false),
            Some(current) => (current.child0, current.child1, current.is_end),
        };
        if bit == 0 {
            let new_child0 = trie_insert(child0, key, idx + 1);
            Some(Box::new(TrieNode { child0: new_child0, child1, is_end }))
        } else {
            let new_child1 = trie_insert(child1, key, idx + 1);
            Some(Box::new(TrieNode { child0, child1: new_child1, is_end }))
        }
    }
}

/// Return `true` iff the suffix `key[idx..]` is a full key stored in the trie.
pub fn trie_search(node: &Option<Box<TrieNode>>, key: &[usize], idx: usize) -> bool {
    if idx == key.len() {
        match node {
            None => false,
            Some(current) => current.is_end,
        }
    } else {
        let bit = key[idx];
        match node {
            None => false,
            Some(current) => {
                if bit == 0 {
                    trie_search(&current.child0, key, idx + 1)
                } else {
                    trie_search(&current.child1, key, idx + 1)
                }
            }
        }
    }
}
