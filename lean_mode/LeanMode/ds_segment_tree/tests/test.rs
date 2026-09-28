use ds_segment_tree::{segtree_build, point_update, range_sum};

#[test]
fn test_build_range_sum_full() {
    let a = [1, 2, 3, 4, 5];
    let st = segtree_build(&a);
    assert_eq!(range_sum(&st, 0, 4), 15);
    assert_eq!(range_sum(&st, 0, 0), 1);
    assert_eq!(range_sum(&st, 4, 4), 5);
    assert_eq!(range_sum(&st, 1, 3), 9);
}

#[test]
fn test_point_update() {
    let a = [1, 2, 3, 4, 5];
    let st0 = segtree_build(&a);
    let st1 = point_update(st0, 2, 100);
    assert_eq!(range_sum(&st1, 0, 4), 112);
    assert_eq!(range_sum(&st1, 2, 2), 100);
    assert_eq!(range_sum(&st1, 0, 1), 3);
}

#[test]
fn test_non_power_of_two() {
    // n = 6, next pow2 = 8, padded leaves are zero
    let a = [1, 1, 1, 1, 1, 1];
    let st = segtree_build(&a);
    assert_eq!(range_sum(&st, 0, 5), 6);
    assert_eq!(range_sum(&st, 3, 5), 3);
}

#[test]
fn test_single_element() {
    let a = [42];
    let st = segtree_build(&a);
    assert_eq!(range_sum(&st, 0, 0), 42);
    let st1 = point_update(st, 0, 7);
    assert_eq!(range_sum(&st1, 0, 0), 7);
}
