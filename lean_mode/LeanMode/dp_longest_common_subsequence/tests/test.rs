// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_basic() {
    let a = vec![1, 2, 3, 4, 5];
    let b = vec![1, 3, 5];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 3);
}

#[test]
fn test_no_common() {
    let a = vec![1, 2, 3];
    let b = vec![4, 5, 6];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 0);
}

#[test]
fn test_identical() {
    let a = vec![1, 2, 3, 4];
    let b = vec![1, 2, 3, 4];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 4);
}

#[test]
fn test_empty_first() {
    let a: Vec<usize> = vec![];
    let b = vec![1, 2, 3];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 0);
}

#[test]
fn test_empty_second() {
    let a = vec![1, 2, 3];
    let b: Vec<usize> = vec![];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 0);
}

#[test]
fn test_empty_both() {
    let a: Vec<usize> = vec![];
    let b: Vec<usize> = vec![];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 0);
}

#[test]
fn test_repeated_elements() {
    let a = vec![1, 2, 2, 3];
    let b = vec![1, 2, 3];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 3);
}

#[test]
fn test_unordered() {
    // LCS = [1, 2, 4] (length 3)
    let a = vec![3, 1, 2, 4];
    let b = vec![1, 2, 3, 4];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 3);
}

#[test]
fn test_single_element() {
    let a = vec![1];
    let b = vec![1];
    assert_eq!(dp_longest_common_subsequence::lcs(&a, &b), 1);
}
