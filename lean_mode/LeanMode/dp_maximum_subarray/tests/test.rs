// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_example_1() {
    let nums = vec![-2, 1, -3, 4, -1, 2, 1, -5, 4];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), 6);
}

#[test]
fn test_all_positive() {
    let nums = vec![1, 2, 3, 4, 5];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), 15);
}

#[test]
fn test_single_negative() {
    let nums = vec![-1];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), -1);
}

#[test]
fn test_single_positive() {
    let nums = vec![5];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), 5);
}

#[test]
fn test_all_negative() {
    let nums = vec![-2, -3, -1, -5];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), -1);
}

#[test]
fn test_zeros() {
    let nums = vec![0, 0, 0];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), 0);
}

#[test]
fn test_mixed() {
    // [2, 3, -4, 5] = 6
    let nums = vec![-1, 2, 3, -4, 5];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), 6);
}

#[test]
fn test_positive_negative_positive() {
    let nums = vec![3, -1, 4];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), 6);
}

#[test]
fn test_large_alternating() {
    let nums = vec![1, -1, 1, -1, 1];
    assert_eq!(dp_maximum_subarray::max_subarray_sum(&nums), 1);
}
