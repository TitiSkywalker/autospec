#[test]
fn test_identical() {
    let a = vec![1, 2, 3, 4];
    let b = vec![1, 2, 3, 4];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 0);
}

#[test]
fn test_substitution() {
    let a = vec![1, 2, 3];
    let b = vec![1, 2, 4];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 1);
}

#[test]
fn test_deletion() {
    let a = vec![1, 2, 3];
    let b = vec![1, 2];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 1);
}

#[test]
fn test_insertion() {
    let a = vec![1, 2];
    let b = vec![1, 2, 3];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 1);
}

#[test]
fn test_empty_to_nonempty() {
    let a: Vec<usize> = vec![];
    let b = vec![1, 2, 3];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 3);
}

#[test]
fn test_nonempty_to_empty() {
    let a = vec![1, 2, 3];
    let b: Vec<usize> = vec![];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 3);
}

#[test]
fn test_both_empty() {
    let a: Vec<usize> = vec![];
    let b: Vec<usize> = vec![];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 0);
}

#[test]
fn test_transposition_like() {
    // [2, 1, 3] -> [1, 2, 3] requires two edits (delete + insert, no transpose).
    let a = vec![2, 1, 3];
    let b = vec![1, 2, 3];
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 2);
}

#[test]
fn test_kitten_sitting() {
    // "kitten" -> "sitting" is the classic Levenshtein distance 3.
    let a = vec![11, 9, 20, 20, 5, 14]; // k i t t e n
    let b = vec![19, 9, 20, 20, 9, 14, 7]; // s i t t i n g
    assert_eq!(dp_edit_distance::edit_distance(&a, &b), 3);
}
