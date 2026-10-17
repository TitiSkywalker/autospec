#[test]
fn test_empty() {
    let out = tensor_relu::relu(vec![]);
    assert!(out.is_empty());
}

#[test]
fn test_single_positive() {
    let out = tensor_relu::relu(vec![3.0]);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 3.0).abs() < 1e-9);
}

#[test]
fn test_single_negative() {
    let out = tensor_relu::relu(vec![-2.5]);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 0.0).abs() < 1e-9);
}

#[test]
fn test_single_zero() {
    let out = tensor_relu::relu(vec![0.0]);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 0.0).abs() < 1e-9);
}

#[test]
fn test_all_positive() {
    let out = tensor_relu::relu(vec![1.0, 2.0, 3.0, 0.5]);
    assert_eq!(out.len(), 4);
    let expected = [1.0, 2.0, 3.0, 0.5];
    for i in 0..4 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_all_negative() {
    let out = tensor_relu::relu(vec![-1.0, -2.0, -3.0]);
    assert_eq!(out.len(), 3);
    for i in 0..3 {
        assert!((out[i] - 0.0).abs() < 1e-9);
    }
}

#[test]
fn test_mixed() {
    let out = tensor_relu::relu(vec![1.0, -0.5, 0.0, 3.0, -2.0]);
    assert_eq!(out.len(), 5);
    let expected = [1.0, 0.0, 0.0, 3.0, 0.0];
    for i in 0..5 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}