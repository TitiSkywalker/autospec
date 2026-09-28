use ds_fenwick::{bit_new, update, query};

#[test]
fn test_empty_query_zero() {
    let b = bit_new(10);
    assert_eq!(query(&b, 0), 0);
    assert_eq!(query(&b, 9), 0);
}

#[test]
fn test_single_update() {
    let b0 = bit_new(5);
    let b1 = update(b0, 2, 7);
    assert_eq!(query(&b1, 0), 0);
    assert_eq!(query(&b1, 1), 0);
    assert_eq!(query(&b1, 2), 7);
    assert_eq!(query(&b1, 4), 7);
}

#[test]
fn test_multiple_updates_prefix_sum() {
    let b0 = bit_new(5);
    let b1 = update(b0, 0, 3);
    let b2 = update(b1, 1, 4);
    let b3 = update(b2, 2, 5);
    let b4 = update(b3, 3, 2);
    // prefix sums: 3, 7, 12, 14, 14
    assert_eq!(query(&b4, 0), 3);
    assert_eq!(query(&b4, 1), 7);
    assert_eq!(query(&b4, 2), 12);
    assert_eq!(query(&b4, 3), 14);
    assert_eq!(query(&b4, 4), 14);
}

#[test]
fn test_update_accumulates() {
    let b0 = bit_new(5);
    let b1 = update(b0, 2, 5);
    let b2 = update(b1, 2, 3);
    assert_eq!(query(&b2, 2), 8);
}
