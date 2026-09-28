#[test]
fn test_insert_and_search() {
    let t0 = ds_trie::trie_insert(None, &[0usize, 1, 0], 0);
    let t1 = ds_trie::trie_insert(t0, &[0usize, 1, 1], 0);
    assert_eq!(ds_trie::trie_search(&t1, &[0usize, 1, 0], 0), true);
    assert_eq!(ds_trie::trie_search(&t1, &[0usize, 1, 1], 0), true);
    assert_eq!(ds_trie::trie_search(&t1, &[0usize, 1], 0), false);
    assert_eq!(ds_trie::trie_search(&t1, &[1usize, 0], 0), false);
}

#[test]
fn test_empty_key() {
    let t0 = ds_trie::trie_insert(None, &[], 0);
    assert_eq!(ds_trie::trie_search(&t0, &[], 0), true);
}

#[test]
fn test_prefix_not_full_key() {
    // insert [0, 0]; searching the prefix [0] must be false (is_end not set there)
    let t0 = ds_trie::trie_insert(None, &[0usize, 0], 0);
    assert_eq!(ds_trie::trie_search(&t0, &[0usize, 0], 0), true);
    assert_eq!(ds_trie::trie_search(&t0, &[0usize], 0), false);
    assert_eq!(ds_trie::trie_search(&t0, &[0usize, 0, 0], 0), false);
}

#[test]
fn test_shared_prefix() {
    // [1] and [1, 0] share the prefix [1]
    let t0 = ds_trie::trie_insert(None, &[1usize], 0);
    let t1 = ds_trie::trie_insert(t0, &[1usize, 0], 0);
    assert_eq!(ds_trie::trie_search(&t1, &[1usize], 0), true);
    assert_eq!(ds_trie::trie_search(&t1, &[1usize, 0], 0), true);
    assert_eq!(ds_trie::trie_search(&t1, &[1usize, 1], 0), false);
    assert_eq!(ds_trie::trie_search(&t1, &[0usize], 0), false);
}

#[test]
fn test_empty_trie_search() {
    assert_eq!(ds_trie::trie_search(&None, &[0usize], 0), false);
    assert_eq!(ds_trie::trie_search(&None, &[], 0), false);
}

#[test]
fn test_longer_key() {
    let t0 = ds_trie::trie_insert(None, &[1usize, 0, 1, 1, 0], 0);
    assert_eq!(ds_trie::trie_search(&t0, &[1usize, 0, 1, 1, 0], 0), true);
    assert_eq!(ds_trie::trie_search(&t0, &[1usize, 0, 1, 1], 0), false);
    assert_eq!(ds_trie::trie_search(&t0, &[1usize, 0, 1, 1, 1], 0), false);
}
