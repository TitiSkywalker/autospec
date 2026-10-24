use ds_lct::{lct_build, lct_link, lct_cut, lct_findroot, lct_path_sum};

#[test]
fn test_link_path_sum() {
    let vals = vec![1, 2, 3, 4, 5];
    let mut t = lct_build(5, &vals);
    t = lct_link(t, 1, 2);
    t = lct_link(t, 2, 3);
    t = lct_link(t, 3, 4);
    let (t, s) = lct_path_sum(t, 1, 4);
    assert_eq!(s, 10);
    let (_t, s2) = lct_path_sum(t, 2, 4);
    assert_eq!(s2, 9);
}

#[test]
fn test_findroot() {
    let vals = vec![7, 8, 9];
    let mut t = lct_build(3, &vals);
    t = lct_link(t, 1, 2);
    t = lct_link(t, 2, 3);
    let (_t, r) = lct_findroot(t, 1);
    assert_eq!(r, 3);
}

#[test]
fn test_cut() {
    let vals = vec![5, 6, 7];
    let mut t = lct_build(3, &vals);
    t = lct_link(t, 1, 2);
    t = lct_link(t, 2, 3);
    t = lct_cut(t, 1, 2);
    // now 1 is isolated, 2-3 are connected
    let (t, r1) = lct_findroot(t, 1);
    let (t, r2) = lct_findroot(t, 2);
    let (_t, r3) = lct_findroot(t, 3);
    assert_eq!(r1, 1);
    assert_eq!(r2, r3);
    assert!(r1 != r2);
}

#[test]
fn test_path_sum_chain() {
    let vals = vec![10, 20, 30];
    let mut t = lct_build(3, &vals);
    t = lct_link(t, 1, 2);
    t = lct_link(t, 2, 3);
    let (t, s) = lct_path_sum(t, 1, 3);
    assert_eq!(s, 60);
    let (_t, s2) = lct_path_sum(t, 3, 1);
    assert_eq!(s2, 60);
}
