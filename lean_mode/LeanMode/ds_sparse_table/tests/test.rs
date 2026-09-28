use ds_sparse_table::{sparse_build, range_min};

#[test]
fn test_full_range() {
    let a = [3, 1, 4, 1, 5, 9, 2];
    let st = sparse_build(&a);
    assert_eq!(range_min(&st, 0, 6), 1);
    assert_eq!(range_min(&st, 0, 0), 3);
    assert_eq!(range_min(&st, 6, 6), 2);
}

#[test]
fn test_subranges() {
    let a = [5, 3, 8, 2, 9, 1, 7];
    let st = sparse_build(&a);
    assert_eq!(range_min(&st, 0, 3), 2);
    assert_eq!(range_min(&st, 2, 5), 1);
    assert_eq!(range_min(&st, 1, 4), 2);
    assert_eq!(range_min(&st, 4, 6), 1);
}

#[test]
fn test_negative_values() {
    let a = [-4, 2, -7, 1, -3];
    let st = sparse_build(&a);
    assert_eq!(range_min(&st, 0, 4), -7);
    assert_eq!(range_min(&st, 0, 1), -4);
    assert_eq!(range_min(&st, 1, 3), -7);
}

#[test]
fn test_single_element() {
    let a = [42];
    let st = sparse_build(&a);
    assert_eq!(range_min(&st, 0, 0), 42);
}

#[test]
fn test_non_power_of_two() {
    let a = [9, 1, 6, 2, 8];
    let st = sparse_build(&a);
    assert_eq!(range_min(&st, 0, 4), 1);
    assert_eq!(range_min(&st, 2, 4), 2);
}
