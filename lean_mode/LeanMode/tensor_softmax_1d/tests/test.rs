#[test]
fn test_uniform() {
    // All equal values → uniform output 1/n
    let input = vec![2.0, 2.0, 2.0, 2.0];
    let out = tensor_softmax_1d::softmax(&input);
    assert_eq!(out.len(), 4);
    for &v in &out {
        assert!((v - 0.25).abs() < 1e-9);
    }
}

#[test]
fn test_single_element() {
    let input = vec![5.0];
    let out = tensor_softmax_1d::softmax(&input);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 1.0).abs() < 1e-9);
}

#[test]
fn test_empty() {
    let input: Vec<f64> = vec![];
    let out = tensor_softmax_1d::softmax(&input);
    assert!(out.is_empty());
}

#[test]
fn test_sum_to_one() {
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let out = tensor_softmax_1d::softmax(&input);
    let sum: f64 = out.iter().sum();
    assert!((sum - 1.0).abs() < 1e-9);
    // Monotonicity: larger input → larger output
    for i in 1..out.len() {
        assert!(out[i] > out[i - 1]);
    }
}

#[test]
fn test_basic() {
    // softmax([1, 2, 3]):
    // sum = e¹ + e² + e³ ≈ 30.192875
    let input = vec![1.0, 2.0, 3.0];
    let out = tensor_softmax_1d::softmax(&input);
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
