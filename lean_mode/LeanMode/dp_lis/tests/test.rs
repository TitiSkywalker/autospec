#[test]
fn test_classic() {
    let nums = vec![10, 9, 2, 5, 3, 7, 101, 18];
    assert_eq!(dp_lis::lis(&nums), 4);
}

#[test]
fn test_already_sorted() {
    let nums = vec![1, 2, 3, 4, 5];
    assert_eq!(dp_lis::lis(&nums), 5);
}

#[test]
fn test_reverse_sorted() {
    let nums = vec![5, 4, 3, 2, 1];
    assert_eq!(dp_lis::lis(&nums), 1);
}

#[test]
fn test_empty() {
    let nums: Vec<i32> = vec![];
    assert_eq!(dp_lis::lis(&nums), 0);
}

#[test]
fn test_single() {
    let nums = vec![42];
    assert_eq!(dp_lis::lis(&nums), 1);
}

#[test]
fn test_all_equal() {
    // Strictly increasing, so duplicates cannot extend the subsequence.
    let nums = vec![2, 2, 2, 2];
    assert_eq!(dp_lis::lis(&nums), 1);
}

#[test]
fn test_negative() {
    let nums = vec![-3, -1, 0, 2];
    assert_eq!(dp_lis::lis(&nums), 4);
}

#[test]
fn test_mixed_sign() {
    let nums = vec![0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15];
    assert_eq!(dp_lis::lis(&nums), 6);
}
