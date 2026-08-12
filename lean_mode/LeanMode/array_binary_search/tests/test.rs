/// Test: target present — unique element in the middle
#[test]
fn test_target_present_middle() {
    let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result = array_binary_search::binary_search_lower_bound(&input, 5);
    assert_eq!(result, 4);
}

/// Test: target present — first element
#[test]
fn test_target_present_first() {
    let input = vec![1, 2, 3, 4, 5];
    let result = array_binary_search::binary_search_lower_bound(&input, 1);
    assert_eq!(result, 0);
}

/// Test: target present — last element
#[test]
fn test_target_present_last() {
    let input = vec![1, 2, 3, 4, 5];
    let result = array_binary_search::binary_search_lower_bound(&input, 5);
    assert_eq!(result, 4);
}

/// Test: target not present — would be in the middle
#[test]
fn test_target_absent_middle() {
    let input = vec![1, 3, 5, 7, 9];
    let result = array_binary_search::binary_search_lower_bound(&input, 4);
    assert_eq!(result, 2);
}

/// Test: target not present — smaller than all elements
#[test]
fn test_target_absent_too_small() {
    let input = vec![10, 20, 30, 40, 50];
    let result = array_binary_search::binary_search_lower_bound(&input, 5);
    assert_eq!(result, 0);
}

/// Test: target not present — larger than all elements
#[test]
fn test_target_absent_too_large() {
    let input = vec![10, 20, 30, 40, 50];
    let result = array_binary_search::binary_search_lower_bound(&input, 100);
    assert_eq!(result, 5);
}

/// Test: duplicates — returns the first occurrence
#[test]
fn test_duplicates_first_occurrence() {
    let input = vec![1, 2, 2, 2, 3, 4, 5];
    let result = array_binary_search::binary_search_lower_bound(&input, 2);
    assert_eq!(result, 1);
}

/// Test: edge case — empty array
#[test]
fn test_edge_case_empty() {
    let input: Vec<i32> = vec![];
    let result = array_binary_search::binary_search_lower_bound(&input, 42);
    assert_eq!(result, 0);
}

/// Test: edge case — single element, target present
#[test]
fn test_edge_case_single_present() {
    let input = vec![42];
    let result = array_binary_search::binary_search_lower_bound(&input, 42);
    assert_eq!(result, 0);
}

/// Test: edge case — single element, target smaller
#[test]
fn test_edge_case_single_smaller() {
    let input = vec![42];
    let result = array_binary_search::binary_search_lower_bound(&input, 10);
    assert_eq!(result, 0);
}

/// Test: edge case — single element, target larger
#[test]
fn test_edge_case_single_larger() {
    let input = vec![42];
    let result = array_binary_search::binary_search_lower_bound(&input, 100);
    assert_eq!(result, 1);
}

/// Test: all elements equal to target
#[test]
fn test_all_equal_to_target() {
    let input = vec![7, 7, 7, 7, 7];
    let result = array_binary_search::binary_search_lower_bound(&input, 7);
    assert_eq!(result, 0);
}

/// Test: all elements less than target
#[test]
fn test_all_less_than_target() {
    let input = vec![1, 2, 3, 4, 5];
    let result = array_binary_search::binary_search_lower_bound(&input, 10);
    assert_eq!(result, 5);
}

/// Test: all elements greater than target
#[test]
fn test_all_greater_than_target() {
    let input = vec![10, 20, 30, 40, 50];
    let result = array_binary_search::binary_search_lower_bound(&input, 0);
    assert_eq!(result, 0);
}

/// Test: negative numbers
#[test]
fn test_negative_numbers() {
    let input = vec![-10, -5, 0, 5, 10];
    let result = array_binary_search::binary_search_lower_bound(&input, -3);
    assert_eq!(result, 2);
}

/// Test: target in a larger sorted array
#[test]
fn test_larger_sorted_array() {
    let input = vec![
        10, 20, 30, 40, 50, 60, 70, 80, 90, 100,
        110, 120, 130, 140, 150, 160, 170, 180, 190, 200,
    ];
    let result = array_binary_search::binary_search_lower_bound(&input, 135);
    assert_eq!(result, 13);
}
