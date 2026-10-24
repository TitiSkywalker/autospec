use ds_seg_beats::{segbeats_build, segbeats_chmin, segbeats_query_max, segbeats_query_sum};

#[test]
fn test_chmin_max_sum() {
    let arr = vec![1, 5, 3, 9, 2, 7, 4, 8];
    let st = segbeats_build(&arr);
    let st = segbeats_chmin(st, 1, 5, 4);
    let (st, mx) = segbeats_query_max(st, 0, 7);
    assert_eq!(mx, 8);
    let (st, mx2) = segbeats_query_max(st, 1, 5);
    assert_eq!(mx2, 4);
    let (_st, sm) = segbeats_query_sum(st, 1, 5);
    assert_eq!(sm, 17);
}

#[test]
fn test_query_without_chmin() {
    let arr = vec![3, 1, 4, 1, 5, 9, 2, 6];
    let st = segbeats_build(&arr);
    let (st, mx) = segbeats_query_max(st, 0, 7);
    assert_eq!(mx, 9);
    let (_st, sm) = segbeats_query_sum(st, 2, 5);
    assert_eq!(sm, 19);
}

#[test]
fn test_chmin_full_range() {
    let arr = vec![10, 20, 30, 40];
    let st = segbeats_build(&arr);
    let st = segbeats_chmin(st, 0, 3, 25);
    let (st, mx) = segbeats_query_max(st, 0, 3);
    assert_eq!(mx, 25);
    let (_st, sm) = segbeats_query_sum(st, 0, 3);
    assert_eq!(sm, 80);
}

#[test]
fn test_chmin_twice() {
    let arr = vec![5, 9, 3, 7, 2];
    let st = segbeats_build(&arr);
    let st = segbeats_chmin(st, 0, 4, 6);
    let st = segbeats_chmin(st, 1, 3, 4);
    let (st, mx) = segbeats_query_max(st, 0, 4);
    assert_eq!(mx, 5);
    let (_st, sm) = segbeats_query_sum(st, 0, 4);
    // after chmin(..,6): [5,6,3,6,2]; after chmin(1..3,4): [5,4,3,4,2] → sum 18
    assert_eq!(sm, 18);
}

#[test]
fn test_chmin_single() {
    let arr = vec![7, 1, 8];
    let st = segbeats_build(&arr);
    let st = segbeats_chmin(st, 2, 2, 3);
    let (st, mx) = segbeats_query_max(st, 0, 2);
    assert_eq!(mx, 7);
    let (_st, sm) = segbeats_query_sum(st, 0, 2);
    assert_eq!(sm, 11);
}
