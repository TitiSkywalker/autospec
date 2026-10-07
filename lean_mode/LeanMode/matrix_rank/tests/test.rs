/// n = 0: empty matrix has rank 0.
#[test]
fn test_rank_empty() {
    assert_eq!(matrix_rank::rank(&[], 0, 0), 0);
}

/// 1×1 non-zero → rank 1.
#[test]
fn test_rank_1x1_nonzero() {
    let m = vec![5.0];
    assert_eq!(matrix_rank::rank(&m, 1, 1), 1);
}

/// 1×1 zero → rank 0.
#[test]
fn test_rank_1x1_zero() {
    let m = vec![0.0];
    assert_eq!(matrix_rank::rank(&m, 1, 1), 0);
}

/// 2×2 singular (row2 = 2·row1) → rank 1.
#[test]
fn test_rank_2x2_singular() {
    let m = vec![1.0, 2.0, 2.0, 4.0];
    assert_eq!(matrix_rank::rank(&m, 2, 2), 1);
}

/// 3×3 identity → rank 3.
#[test]
fn test_rank_3x3_identity() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 3, 3), 3);
}

/// 3×3 full rank (det = -3) → rank 3.
#[test]
fn test_rank_3x3_full() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 10.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 3, 3), 3);
}

/// 3×3 with two identical rows → rank 2.
#[test]
fn test_rank_3x3_rank2() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        1.0, 2.0, 3.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 3, 3), 2);
}

/// 3×3 with three identical rows → rank 1.
#[test]
fn test_rank_3x3_rank1() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        1.0, 2.0, 3.0,
        1.0, 2.0, 3.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 3, 3), 1);
}

/// 3×3 with an all-zero column → rank 2.
#[test]
fn test_rank_3x3_zero_column() {
    #[rustfmt::skip]
    let m = vec![
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
        0.0, 1.0, 1.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 3, 3), 2);
}

/// 3×3 zero matrix → rank 0.
#[test]
fn test_rank_3x3_zero() {
    #[rustfmt::skip]
    let m = vec![
        0.0, 0.0, 0.0,
        0.0, 0.0, 0.0,
        0.0, 0.0, 0.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 3, 3), 0);
}

/// 3×2 tall matrix (two independent columns) → rank 2.
#[test]
fn test_rank_3x2_tall() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 0.0,
        0.0, 1.0,
        0.0, 0.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 3, 2), 2);
}

/// 2×3 wide matrix (two independent rows) → rank 2.
#[test]
fn test_rank_2x3_wide() {
    #[rustfmt::skip]
    let m = vec![
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
    ];
    assert_eq!(matrix_rank::rank(&m, 2, 3), 2);
}
