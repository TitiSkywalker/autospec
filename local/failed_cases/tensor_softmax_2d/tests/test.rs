#[test]
fn test_uniform() {
    // All equal values → uniform output 1/cols per element in each row
    let input = vec![2.0, 2.0, 2.0, 2.0, 2.0, 2.0];
    let out = tensor_softmax_2d::softmax2d(&input, 2, 3);
    assert_eq!(out.len(), 6);
    for &v in &out {
        assert!((v - 1.0 / 3.0).abs() < 1e-9);
    }
}

#[test]
fn test_single_row() {
    // Single row: equivalent to 1D softmax
    let input = vec![1.0, 2.0, 3.0];
    let out = tensor_softmax_2d::softmax2d(&input, 1, 3);
    assert_eq!(out.len(), 3);
    let sum = std::f64::consts::E + std::f64::consts::E.powi(2) + std::f64::consts::E.powi(3);
    let expected = [
        std::f64::consts::E / sum,
        std::f64::consts::E.powi(2) / sum,
        std::f64::consts::E.powi(3) / sum,
    ];
    for i in 0..3 {
        assert!((out[i] - expected[i]).abs() < 1e-6);
    }
}

#[test]
fn test_single_column() {
    // Each row has 1 element → softmax of a single value = 1.0
    let input = vec![5.0, 3.0, 7.0];
    let out = tensor_softmax_2d::softmax2d(&input, 3, 1);
    assert_eq!(out.len(), 3);
    for &v in &out {
        assert!((v - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_empty() {
    let out = tensor_softmax_2d::softmax2d(&[], 0, 0);
    assert!(out.is_empty());
    let out2 = tensor_softmax_2d::softmax2d(&[1.0, 2.0], 1, 0);
    assert!(out2.is_empty());
    let out3 = tensor_softmax_2d::softmax2d(&[1.0, 2.0], 0, 2);
    assert!(out3.is_empty());
}

#[test]
fn test_sum_to_one_per_row() {
    // Each row should sum to 1.0
    let input = vec![
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        0.0, 1.0, 2.0, 3.0,
    ];
    let out = tensor_softmax_2d::softmax2d(&input, 3, 4);
    assert_eq!(out.len(), 12);
    for r in 0..3 {
        let row_sum: f64 = out[r * 4..(r + 1) * 4].iter().sum();
        assert!((row_sum - 1.0).abs() < 1e-9);
    }
    // Monotonicity within each row: larger input → larger output
    for r in 0..3 {
        let row_start = r * 4;
        for c in 1..4 {
            assert!(
                out[row_start + c] > out[row_start + c - 1],
                "Row {}: out[{}] should be > out[{}]",
                r,
                c,
                c - 1
            );
        }
    }
}

#[test]
fn test_basic() {
    // 2x3 input:
    // Row 0: [1, 2, 3]
    // Row 1: [4, 5, 6]
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let out = tensor_softmax_2d::softmax2d(&input, 2, 3);
    assert_eq!(out.len(), 6);

    // Row 0: sum = e¹ + e² + e³
    let sum0 = std::f64::consts::E + std::f64::consts::E.powi(2) + std::f64::consts::E.powi(3);
    let expected0 = [
        std::f64::consts::E / sum0,
        std::f64::consts::E.powi(2) / sum0,
        std::f64::consts::E.powi(3) / sum0,
    ];
    for j in 0..3 {
        assert!((out[j] - expected0[j]).abs() < 1e-6);
    }

    // Row 1: sum = e⁴ + e⁵ + e⁶
    let sum1 =
        std::f64::consts::E.powi(4) + std::f64::consts::E.powi(5) + std::f64::consts::E.powi(6);
    let expected1 = [
        std::f64::consts::E.powi(4) / sum1,
        std::f64::consts::E.powi(5) / sum1,
        std::f64::consts::E.powi(6) / sum1,
    ];
    for j in 0..3 {
        assert!((out[3 + j] - expected1[j]).abs() < 1e-6);
    }
}

#[test]
fn test_rows_independent() {
    // Verify rows are independent: changing one row shouldn't affect others
    // Row 0: [1, 2] → softmax
    // Row 1: [100, 1] → first element dominates
    let input = vec![1.0, 2.0, 100.0, 1.0];
    let out = tensor_softmax_2d::softmax2d(&input, 2, 2);
    assert_eq!(out.len(), 4);

    // Row 0 should be softmax of [1, 2]
    let sum0 = std::f64::consts::E + std::f64::consts::E.powi(2);
    assert!((out[0] - std::f64::consts::E / sum0).abs() < 1e-9);
    assert!((out[1] - std::f64::consts::E.powi(2) / sum0).abs() < 1e-9);

    // Row 1: first element should be close to 1, second close to 0
    assert!(out[2] > 0.999);
    assert!(out[3] < 0.001);
}
