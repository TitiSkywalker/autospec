#[test]
fn test_empty() {
    let result = array_permutations::permutations(0);
    assert_eq!(result, vec![]);
}

#[test]
fn test_single() {
    let result = array_permutations::permutations(1);
    assert_eq!(result, vec![0]);
}

#[test]
fn test_two() {
    let result = array_permutations::permutations(2);
    assert_eq!(result, vec![0, 1, 1, 0]);
}

#[test]
fn test_three() {
    let result = array_permutations::permutations(3);
    assert_eq!(
        result,
        vec![0, 1, 2, 0, 2, 1, 1, 0, 2, 1, 2, 0, 2, 0, 1, 2, 1, 0]
    );
}

#[test]
fn test_four_count_and_shape() {
    let result = array_permutations::permutations(4);
    // 4! = 24 permutations, each of length 4.
    assert_eq!(result.len(), 96);
    // The first block is [0, 1, 2, 3]; the last block is [3, 2, 1, 0].
    assert_eq!(result[0], 0);
    assert_eq!(result[1], 1);
    assert_eq!(result[2], 2);
    assert_eq!(result[3], 3);
    assert_eq!(result[92], 3);
    assert_eq!(result[93], 2);
    assert_eq!(result[94], 1);
    assert_eq!(result[95], 0);
}
