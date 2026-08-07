/// Test: basic known-answer — 10 elements
#[test]
fn test_known_answer_basic() {
    let input = vec![10.0, 8.0, 4.0, 3.0, 1.0, 9.0, 2.0, 7.0, 5.0, 6.0];
    let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: known-answer with float values
#[test]
fn test_known_answer_floats() {
    let input = vec![1.5, 2.3, 0.5, 3.1, 1.2];
    let expected = vec![0.5, 1.2, 1.5, 2.3, 3.1];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: edge case — empty array
#[test]
fn test_edge_case_empty() {
    let input: Vec<f64> = vec![];
    let expected: Vec<f64> = vec![];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: edge case — single element
#[test]
fn test_edge_case_single_element() {
    let input = vec![42.0];
    let expected = vec![42.0];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: edge case — already sorted
#[test]
fn test_edge_case_pre_sorted() {
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: edge case — reverse sorted
#[test]
fn test_edge_case_reverse_sorted() {
    let input = vec![5.0, 4.0, 3.0, 2.0, 1.0];
    let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: edge case — all same elements
#[test]
fn test_edge_case_all_same_elements() {
    let input = vec![7.0, 7.0, 7.0, 7.0, 7.0];
    let expected = vec![7.0, 7.0, 7.0, 7.0, 7.0];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

/// Test: known-answer with duplicates
#[test]
fn test_known_answer_with_duplicates() {
    let input = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0, 5.0, 3.0];
    let expected = vec![1.0, 1.0, 2.0, 3.0, 3.0, 4.0, 5.0, 5.0, 6.0, 9.0];
    let result = array_bubble_sort::bubble_sort(&input);
    let tol = 0.001;
    for i in 0..result.len() {
        let diff = result[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}
