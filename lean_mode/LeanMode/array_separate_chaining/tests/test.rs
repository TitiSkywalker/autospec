use array_separate_chaining::{ct_contains, ct_insert, ct_new};

#[test]
fn test_insert_new() {
    let t0 = ct_new(5);
    let (t1, b1) = ct_insert(t0, 12);
    assert!(b1);
    let (t2, b2) = ct_insert(t1, 7);
    assert!(b2);
    let (t3, b3) = ct_insert(t2, 22);
    assert!(b3);
    let (t4, b4) = ct_insert(t3, 3);
    assert!(b4);
    assert!(ct_contains(&t4, 12));
    assert!(ct_contains(&t4, 7));
    assert!(ct_contains(&t4, 22));
    assert!(ct_contains(&t4, 3));
}

#[test]
fn test_insert_duplicate() {
    let t0 = ct_new(5);
    let (t1, b1) = ct_insert(t0, 12);
    assert!(b1);
    let (t2, b2) = ct_insert(t1, 12);
    assert!(!b2);
    assert!(ct_contains(&t2, 12));
}

#[test]
fn test_contains_absent() {
    let t0 = ct_new(5);
    let (t1, _) = ct_insert(t0, 12);
    let (t2, _) = ct_insert(t1, 7);
    assert!(!ct_contains(&t2, 100));
    assert!(!ct_contains(&t2, 13));
    assert!(!ct_contains(&t2, 1));
}
