#[test]
fn test_found_middle() {
    let nums = vec![2, 7, 11, 15];
    assert!(array_two_sum::two_sum(&nums, 9));
}

#[test]
fn test_found_edges() {
    let nums = vec![1, 2, 3, 4, 5];
    assert!(array_two_sum::two_sum(&nums, 6));
}

#[test]
fn test_not_found() {
    let nums = vec![1, 2, 3, 4, 5];
    assert!(!array_two_sum::two_sum(&nums, 10));
}

#[test]
fn test_duplicates() {
    let nums = vec![3, 3];
    assert!(array_two_sum::two_sum(&nums, 6));
}

#[test]
fn test_negative_numbers() {
    let nums = vec![-5, -3, 0, 2, 4];
    assert!(array_two_sum::two_sum(&nums, -1));
}

#[test]
fn test_negative_not_found() {
    let nums = vec![-5, -3, 0, 2, 4];
    assert!(!array_two_sum::two_sum(&nums, 100));
}

#[test]
fn test_empty() {
    let nums: Vec<i32> = vec![];
    assert!(!array_two_sum::two_sum(&nums, 0));
}

#[test]
fn test_single_element() {
    let nums = vec![42];
    assert!(!array_two_sum::two_sum(&nums, 42));
}

#[test]
fn test_zero_pair() {
    let nums = vec![-4, 0, 0, 7];
    assert!(array_two_sum::two_sum(&nums, 0));
}

#[test]
fn test_larger_sorted() {
    let nums = vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    assert!(array_two_sum::two_sum(&nums, 24));
    assert!(!array_two_sum::two_sum(&nums, 2));
}
