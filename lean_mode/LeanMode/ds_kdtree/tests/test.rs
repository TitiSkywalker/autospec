use ds_kdtree::{kd_insert, kd_query, Node};

#[test]
fn test_query_all() {
    let mut t: Option<Box<Node>> = None;
    t = kd_insert(t, 2, 3, 5);
    t = kd_insert(t, 5, 4, 7);
    t = kd_insert(t, 9, 6, 11);
    t = kd_insert(t, 4, 7, 13);
    t = kd_insert(t, 8, 1, 17);
    // [0,10] x [0,10] contains everything → 5+7+11+13+17 = 53
    assert_eq!(kd_query(&t, 0, 0, 10, 10), 53);
}

#[test]
fn test_query_partial() {
    let mut t: Option<Box<Node>> = None;
    t = kd_insert(t, 2, 3, 5);
    t = kd_insert(t, 5, 4, 7);
    t = kd_insert(t, 9, 6, 11);
    t = kd_insert(t, 4, 7, 13);
    t = kd_insert(t, 8, 1, 17);
    // [3,7] x [3,7] contains (5,4)=7 and (4,7)=13 → 20
    assert_eq!(kd_query(&t, 3, 3, 7, 7), 20);
}

#[test]
fn test_query_single_point() {
    let mut t: Option<Box<Node>> = None;
    t = kd_insert(t, 2, 3, 5);
    t = kd_insert(t, 5, 4, 7);
    // exactly the point (5,4) → 7
    assert_eq!(kd_query(&t, 5, 4, 5, 4), 7);
    // exactly the point (2,3) → 5
    assert_eq!(kd_query(&t, 2, 3, 2, 3), 5);
}

#[test]
fn test_query_empty_tree() {
    let t: Option<Box<Node>> = None;
    assert_eq!(kd_query(&t, 0, 0, 100, 100), 0);
}

#[test]
fn test_query_disjoint() {
    let mut t: Option<Box<Node>> = None;
    t = kd_insert(t, 10, 10, 3);
    t = kd_insert(t, 20, 20, 4);
    // rectangle far away → 0
    assert_eq!(kd_query(&t, 100, 100, 200, 200), 0);
    // rectangle covering only the first point → 3
    assert_eq!(kd_query(&t, 0, 0, 15, 15), 3);
}
