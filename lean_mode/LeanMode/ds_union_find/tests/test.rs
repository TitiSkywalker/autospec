use ds_union_find::{dsu_new, find, union, connected};

#[test]
fn test_singleton_roots() {
    let d0 = dsu_new(5);
    let (d1, r0) = find(d0, 0);
    assert_eq!(r0, 0);
    let (_, r3) = find(d1, 3);
    assert_eq!(r3, 3);
}

#[test]
fn test_union_connectivity() {
    let d0 = dsu_new(5);
    let (d1, merged1) = union(d0, 0, 1);
    assert!(merged1);
    let (d2, merged2) = union(d1, 1, 2);
    assert!(merged2);
    assert!(connected(&d2, 0, 2));
    assert!(!connected(&d2, 0, 3));
}

#[test]
fn test_union_same_set_noop() {
    let d0 = dsu_new(5);
    let (d1, m1) = union(d0, 0, 1);
    assert!(m1);
    let (d2, m2) = union(d1, 0, 1);
    assert!(!m2);
    assert!(connected(&d2, 0, 1));
}

#[test]
fn test_transitive_connectivity() {
    let d0 = dsu_new(6);
    let (d1, _) = union(d0, 0, 1);
    let (d2, _) = union(d1, 1, 2);
    let (d3, _) = union(d2, 3, 4);
    let (d4, _) = union(d3, 2, 3);
    assert!(connected(&d4, 0, 4));
    assert!(connected(&d4, 1, 3));
    assert!(!connected(&d4, 0, 5));
}

#[test]
fn test_find_is_idempotent() {
    let d0 = dsu_new(5);
    let (d1, _) = union(d0, 0, 1);
    let (d2, _) = union(d1, 1, 2);
    let (d3, r) = find(d2, 0);
    // find again returns the same root
    let (_, r2) = find(d3, 0);
    assert_eq!(r, r2);
}
