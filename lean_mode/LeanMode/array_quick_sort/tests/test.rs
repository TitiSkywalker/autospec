/// Test: basic known-answer — 10 elements
#[test]
fn test_known_answer_basic() {
    let input = vec![10, 8, 4, 3, 1, 9, 2, 7, 5, 6];
    let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}

/// Test: known-answer with negative integers
#[test]
fn test_known_answer_negatives() {
    let input = vec![-5, 3, -1, 0, 2, -4, 1, -2, 4, -3];
    let expected = vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}

/// Test: edge case — empty array
#[test]
fn test_edge_case_empty() {
    let input: Vec<i32> = vec![];
    let expected: Vec<i32> = vec![];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}

/// Test: edge case — single element
#[test]
fn test_edge_case_single_element() {
    let input = vec![42];
    let expected = vec![42];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}

/// Test: edge case — already sorted
#[test]
fn test_edge_case_pre_sorted() {
    let input = vec![1, 2, 3, 4, 5];
    let expected = vec![1, 2, 3, 4, 5];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}

/// Test: edge case — reverse sorted
#[test]
fn test_edge_case_reverse_sorted() {
    let input = vec![5, 4, 3, 2, 1];
    let expected = vec![1, 2, 3, 4, 5];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}

/// Test: edge case — all same elements
#[test]
fn test_edge_case_all_same_elements() {
    let input = vec![7, 7, 7, 7, 7];
    let expected = vec![7, 7, 7, 7, 7];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}

/// Test: known-answer with duplicates
#[test]
fn test_known_answer_with_duplicates() {
    let input = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
    let expected = vec![1, 1, 2, 3, 3, 4, 5, 5, 6, 9];
    let result = array_quick_sort::quick_sort(&input);
    assert_eq!(result, expected);
}
