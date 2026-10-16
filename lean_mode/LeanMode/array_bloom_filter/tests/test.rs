use array_bloom_filter::{bf_contains, bf_insert, bf_new};

#[test]
fn test_insert_then_contains() {
    let f0 = bf_new(16, 3);
    let f1 = bf_insert(f0, 5);
    assert!(bf_contains(&f1, 5));
}

#[test]
fn test_absent_key() {
    let f0 = bf_new(16, 3);
    let f1 = bf_insert(f0, 5);
    assert!(!bf_contains(&f1, 10));
}

#[test]
fn test_multiple_inserts() {
    let f0 = bf_new(16, 3);
    let f1 = bf_insert(f0, 5);
    let f2 = bf_insert(f1, 10);
    assert!(bf_contains(&f2, 5));
    assert!(bf_contains(&f2, 10));
    assert!(!bf_contains(&f2, 99));
}

#[test]
fn test_insert_idempotent() {
    let f0 = bf_new(16, 3);
    let f1 = bf_insert(f0, 7);
    let f2 = bf_insert(f1, 7);
    assert!(bf_contains(&f2, 7));
    assert!(!bf_contains(&f2, 3));
}
