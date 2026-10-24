#[test]
fn test_single_node() {
    let p1 = vec![0usize];
    let p2 = vec![0usize];
    assert!(rooted_tree_isomorphism::rooted_tree_isomorphism(1, &p1, 0, 1, &p2, 0));
}

#[test]
fn test_path_vs_star() {
    // Path 0-1-2 (root 0) vs star with center 0 and leaves 1, 2.
    let p1 = vec![0usize, 0usize, 1usize];
    let p2 = vec![0usize, 0usize, 0usize];
    let r = rooted_tree_isomorphism::rooted_tree_isomorphism(3, &p1, 0, 3, &p2, 0);
    assert!(!r);
}

#[test]
fn test_relabeled_path() {
    // Path 0-1-2 rooted at 0, vs the same shape with vertices relabelled
    // (root 2, with 0's parent = 2 and 1's parent = 0).
    let p1 = vec![0usize, 0usize, 1usize];
    let p2 = vec![2usize, 0usize, 2usize];
    let r = rooted_tree_isomorphism::rooted_tree_isomorphism(3, &p1, 0, 3, &p2, 2);
    assert!(r);
}

#[test]
fn test_different_sizes() {
    let p1 = vec![0usize];
    let p2 = vec![0usize, 0usize, 1usize];
    let r = rooted_tree_isomorphism::rooted_tree_isomorphism(1, &p1, 0, 3, &p2, 0);
    assert!(!r);
}

#[test]
fn test_path4_vs_broom() {
    // Path 0-1-2-3 vs a "broom": root 0 with children 1, 2, and 1 has child 3.
    let p1 = vec![0usize, 0usize, 1usize, 2usize];
    let p2 = vec![0usize, 0usize, 0usize, 1usize];
    let r = rooted_tree_isomorphism::rooted_tree_isomorphism(4, &p1, 0, 4, &p2, 0);
    assert!(!r);
}

#[test]
fn test_star_same_size() {
    // Two stars of 5 nodes (all leaves attached to the root): isomorphic.
    let p1 = vec![0usize, 0usize, 0usize, 0usize, 0usize];
    let p2 = vec![4usize, 4usize, 4usize, 4usize, 4usize];
    let r = rooted_tree_isomorphism::rooted_tree_isomorphism(5, &p1, 0, 5, &p2, 4);
    assert!(r);
}
