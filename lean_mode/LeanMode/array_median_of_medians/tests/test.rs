use array_median_of_medians::select_kth;

/// Test: sorted array, middle element
#[test]
fn test_sorted_middle() {
    let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result = select_kth(&input, 4);
    assert_eq!(result, 5);
}

/// Test: sorted array, first element
#[test]
fn test_sorted_first() {
    let input = vec![10, 20, 30, 40, 50];
    let result = select_kth(&input, 0);
    assert_eq!(result, 10);
}

/// Test: sorted array, last element
#[test]
fn test_sorted_last() {
    let input = vec![10, 20, 30, 40, 50];
    let result = select_kth(&input, 4);
    assert_eq!(result, 50);
}

/// Test: unsorted array
#[test]
fn test_unsorted() {
    let input = vec![7, 10, 4, 3, 20, 15];
    let result = select_kth(&input, 3);
    assert_eq!(result, 10);
}

/// Test: negative numbers
#[test]
fn test_negative_numbers() {
    let input = vec![-5, -1, -3, 0, 2, -10];
    let result = select_kth(&input, 2);
    assert_eq!(result, -3);
}

/// Test: duplicates — k-th among equal elements
#[test]
fn test_duplicates() {
    let input = vec![4, 4, 4, 4, 4];
    let result = select_kth(&input, 2);
    assert_eq!(result, 4);
}

/// Test: duplicates mixed with distinct values
#[test]
fn test_duplicates_mixed() {
    let input = vec![3, 1, 2, 1, 3, 2];
    let result = select_kth(&input, 4);
    assert_eq!(result, 3);
}

/// Test: single element
#[test]
fn test_single() {
    let input = vec![42];
    let result = select_kth(&input, 0);
    assert_eq!(result, 42);
}

/// Test: large array (> 5 elements, exercises the recursive median path)
#[test]
fn test_large_array() {
    let input = vec![
        51, 3, 84, 27, 96, 14, 62, 7, 38, 71,
        45, 9, 66, 23, 58, 12, 79, 33, 90, 5,
        41, 68, 19, 74, 30, 87, 16, 55, 2, 49,
    ];
    let result = select_kth(&input, 20);
    assert_eq!(result, 62);
}

/// Test: large array, minimum
#[test]
fn test_large_min() {
    let input = vec![
        51, 3, 84, 27, 96, 14, 62, 7, 38, 71,
        45, 9, 66, 23, 58, 12, 79, 33, 90, 5,
    ];
    let result = select_kth(&input, 0);
    assert_eq!(result, 3);
}

/// Test: large array, maximum
#[test]
fn test_large_max() {
    let input = vec![
        51, 3, 84, 27, 96, 14, 62, 7, 38, 71,
        45, 9, 66, 23, 58, 12, 79, 33, 90, 5,
    ];
    let result = select_kth(&input, 19);
    assert_eq!(result, 96);
}

/// Test: reversed array
#[test]
fn test_reversed() {
    let input = vec![100, 90, 80, 70, 60, 50, 40, 30, 20, 10];
    let result = select_kth(&input, 5);
    assert_eq!(result, 60);
}
