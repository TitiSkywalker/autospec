#[test]
fn test_classic_3x3() {
    // [[1,3,1],[1,5,1],[4,2,1]] -> 1+3+1+1+1 = 7
    let grid = vec![1, 3, 1, 1, 5, 1, 4, 2, 1];
    assert_eq!(dp_matrix_path::min_path_sum(&grid, 3, 3), 7);
}

#[test]
fn test_2x3() {
    // [[1,2,3],[4,5,6]] -> 1+2+3+6 = 12
    let grid = vec![1, 2, 3, 4, 5, 6];
    assert_eq!(dp_matrix_path::min_path_sum(&grid, 2, 3), 12);
}

#[test]
fn test_single_cell() {
    let grid = vec![5];
    assert_eq!(dp_matrix_path::min_path_sum(&grid, 1, 1), 5);
}

#[test]
fn test_single_row() {
    let grid = vec![1, 2, 3];
    assert_eq!(dp_matrix_path::min_path_sum(&grid, 1, 3), 6);
}

#[test]
fn test_single_column() {
    let grid = vec![1, 2, 3];
    assert_eq!(dp_matrix_path::min_path_sum(&grid, 3, 1), 6);
}

#[test]
fn test_negative_values() {
    // [[-1,-2],[-3,-4]] -> down then right: -1 + -3 + -4 = -8
    let grid = vec![-1, -2, -3, -4];
    assert_eq!(dp_matrix_path::min_path_sum(&grid, 2, 2), -8);
}

#[test]
fn test_larger() {
    // [[1,4,8,6,2],[2,7,4,3,1],[5,1,9,2,3],[6,3,8,7,4]]
    // Optimal path: 1 -> 2 -> 7 -> 4 -> 3 -> 1 -> 3 -> 4 = 25.
    let grid = vec![
        1, 4, 8, 6, 2,
        2, 7, 4, 3, 1,
        5, 1, 9, 2, 3,
        6, 3, 8, 7, 4,
    ];
    assert_eq!(dp_matrix_path::min_path_sum(&grid, 4, 5), 25);
}
