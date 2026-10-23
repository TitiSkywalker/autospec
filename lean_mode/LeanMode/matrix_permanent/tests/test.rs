//! Tests for the permanent (Ryser's formula).
//!
//! All expected values were computed independently by the brute-force
//! permutation expansion `per(A) = Σ_σ Π_i A[i][σ(i)]`.

/// n = 0: the permanent of the empty matrix is the empty product, 1.
#[test]
fn test_permanent_empty() {
    let m: Vec<i64> = Vec::new();
    assert_eq!(matrix_permanent::permanent(&m, 0), 1);
}

/// n = 1: [[5]] → 5, [[-3]] → -3.
#[test]
fn test_permanent_1x1() {
    let m = vec![5i64];
    assert_eq!(matrix_permanent::permanent(&m, 1), 5);
    let neg = vec![-3i64];
    assert_eq!(matrix_permanent::permanent(&neg, 1), -3);
}

/// n = 2: [[1, 2], [3, 4]] → 1·4 + 2·3 = 10 (the determinant is −2).
#[test]
fn test_permanent_2x2() {
    let m = vec![1i64, 2, 3, 4];
    assert_eq!(matrix_permanent::permanent(&m, 2), 10);
}

/// n = 2: identity → 1, zero matrix → 0.
#[test]
fn test_permanent_2x2_edge() {
    let id = vec![1i64, 0, 0, 1];
    assert_eq!(matrix_permanent::permanent(&id, 2), 1);
    let zero = vec![0i64, 0, 0, 0];
    assert_eq!(matrix_permanent::permanent(&zero, 2), 0);
}

/// n = 2: [[-1, 2], [3, -4]] → 4 + 6 = 10.
#[test]
fn test_permanent_2x2_signed() {
    let m = vec![-1i64, 2, 3, -4];
    assert_eq!(matrix_permanent::permanent(&m, 2), 10);
}

/// n = 3: [[1, 2, 3], [4, 5, 6], [7, 8, 10]] → 463
/// (the determinant of this matrix is −3, so a permanent/determinant mix-up
/// is caught here).
#[test]
fn test_permanent_3x3() {
    let m = vec![1i64, 2, 3, 4, 5, 6, 7, 8, 10];
    assert_eq!(matrix_permanent::permanent(&m, 3), 463);
}

/// n = 3: the all-ones matrix has 3! = 6 permutations, each contributing 1.
#[test]
fn test_permanent_3x3_ones() {
    let m = vec![1i64; 9];
    assert_eq!(matrix_permanent::permanent(&m, 3), 6);
}

/// n = 3: a zero column kills every permutation — permanent is 0.
#[test]
fn test_permanent_3x3_zero_column() {
    let m = vec![1i64, 0, 3, 4, 0, 6, 7, 0, 9];
    assert_eq!(matrix_permanent::permanent(&m, 3), 0);
}

/// n = 3: mixed signs inside the matrix → −90.
#[test]
fn test_permanent_3x3_signed() {
    let m = vec![-1i64, 2, 3, 4, -5, 6, 7, 8, -9];
    assert_eq!(matrix_permanent::permanent(&m, 3), -90);
}

/// n = 3: flipping the sign of two whole rows flips the permanent's sign
/// (a determinant would be unchanged).
#[test]
fn test_permanent_3x3_two_negated_rows() {
    let m = vec![-1i64, -2, -3, -4, -5, -6, -7, -8, -10];
    assert_eq!(matrix_permanent::permanent(&m, 3), -463);
}

/// n = 4: all-ones → 4! = 24.
#[test]
fn test_permanent_4x4_ones() {
    let m = vec![1i64; 16];
    assert_eq!(matrix_permanent::permanent(&m, 4), 24);
}

/// n = 4: identity → 1.
#[test]
fn test_permanent_4x4_identity() {
    #[rustfmt::skip]
    let m = vec![
        1i64, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        0, 0, 0, 1,
    ];
    assert_eq!(matrix_permanent::permanent(&m, 4), 1);
}

/// n = 4: general signed matrix → 18016.
#[test]
fn test_permanent_4x4_signed() {
    #[rustfmt::skip]
    let m = vec![
         1i64, -2,  3,   4,
         5,     6, -7,   8,
         9,    10, 11, -12,
       -13,    14, 15,  16,
    ];
    assert_eq!(matrix_permanent::permanent(&m, 4), 18016);
}

/// n = 4: another signed matrix → 40.
#[test]
fn test_permanent_4x4_signed2() {
    #[rustfmt::skip]
    let m = vec![
        2i64, -1, 0,  3,
        1,     2, -2, 1,
        0,     1, 3, -1,
        4,     0, 1,  2,
    ];
    assert_eq!(matrix_permanent::permanent(&m, 4), 40);
}

/// n = 4: nine negative entries → 8800.
#[test]
fn test_permanent_4x4_many_negatives() {
    #[rustfmt::skip]
    let m = vec![
        -1i64, -2, -3, -4,
        -5,     6, -7,  8,
         9,   -10, 11, -12,
       -13,    14, -15, 16,
    ];
    assert_eq!(matrix_permanent::permanent(&m, 4), 8800);
}

/// n = 5: 0/1 matrix — the permanent counts the perfect matchings → 13.
#[test]
fn test_permanent_5x5_zero_one() {
    #[rustfmt::skip]
    let m = vec![
        1i64, 1, 0, 0, 1,
        1,    1, 1, 0, 0,
        0,    1, 1, 1, 0,
        0,    0, 1, 1, 1,
        1,    0, 0, 1, 1,
    ];
    assert_eq!(matrix_permanent::permanent(&m, 5), 13);
}

/// n = 5: general matrix → 14480700.
#[test]
fn test_permanent_5x5() {
    #[rustfmt::skip]
    let m = vec![
         1i64,  2,  3,  4,  5,
         6,     7,  8,  9, 10,
        11,    12, 13, 14, 15,
        16,    17, 18, 19, 20,
        21,    22, 23, 24, 25,
    ];
    assert_eq!(matrix_permanent::permanent(&m, 5), 14480700);
}
