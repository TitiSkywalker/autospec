use array_open_addressing::{ot_contains, ot_insert, ot_new};

#[test]
fn test_insert_new_keys() {
    let t0 = ot_new(7);
    let (t1, b1) = ot_insert(t0, 10);
    assert!(b1);
    let (t2, b2) = ot_insert(t1, 3);
    assert!(b2);
    let (t3, b3) = ot_insert(t2, 17);
    assert!(b3);
    let (t4, b4) = ot_insert(t3, 24);
    assert!(b4);
    assert!(ot_contains(&t4, 10));
    assert!(ot_contains(&t4, 3));
    assert!(ot_contains(&t4, 17));
    assert!(ot_contains(&t4, 24));
}

#[test]
fn test_insert_duplicate() {
    let t0 = ot_new(7);
    let (t1, b1) = ot_insert(t0, 10);
    assert!(b1);
    let (t2, b2) = ot_insert(t1, 10);
    assert!(!b2);
    assert!(ot_contains(&t2, 10));
}

#[test]
fn test_contains_absent() {
    let t0 = ot_new(7);
    let (t1, _) = ot_insert(t0, 10);
    let (t2, _) = ot_insert(t1, 3);
    let (t3, _) = ot_insert(t2, 17);
    let (t4, _) = ot_insert(t3, 24);
    assert!(!ot_contains(&t4, 5));
    assert!(!ot_contains(&t4, 99));
}

#[test]
fn test_full_table() {
    let t0 = ot_new(5);
    let (t1, b0) = ot_insert(t0, 0);
    assert!(b0);
    let (t2, b1) = ot_insert(t1, 1);
    assert!(b1);
    let (t3, b2) = ot_insert(t2, 2);
    assert!(b2);
    let (t4, b3) = ot_insert(t3, 3);
    assert!(b3);
    let (t5, b4) = ot_insert(t4, 4);
    assert!(b4);
    // Table is full; inserting a new key reports `false` and changes nothing.
    let (t6, b5) = ot_insert(t5, 5);
    assert!(!b5);
    assert!(!ot_contains(&t6, 5));
    assert!(ot_contains(&t6, 3));
}
