#[test]
fn test_identity() {
    // Full convolution with kernel to input [1, 0, 0]
    // input: [3, 1, 4, 1, 5], kernel: [1, 0, 0]
    let input = vec![3.0, 1.0, 4.0, 1.0, 5.0];
    let kernel = vec![1.0, 0.0, 0.0];
    let out = tensor_convolution_1d::conv1d(&input, &kernel);
    assert_eq!(out.len(), 7);
    let expected = vec![3.0, 1.0, 4.0, 1.0, 5.0, 0.0, 0.0];
    for i in 0..out.len() {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_moving_average() {
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let kernel = vec![0.5, 0.5];
    let out = tensor_convolution_1d::conv1d(&input, &kernel);
    assert_eq!(out.len(), 6);
    let expected = vec![0.5, 1.5, 2.5, 3.5, 4.5, 2.5];
    for i in 0..out.len() {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_derivative() {
    // Kernel [1, -1] with full convolution
    let input = vec![1.0, 3.0, 6.0, 10.0];
    let kernel = vec![1.0, -1.0];
    let out = tensor_convolution_1d::conv1d(&input, &kernel);
    assert_eq!(out.len(), 5);
    let expected = vec![1.0, 2.0, 3.0, 4.0, -10.0];
    for i in 0..out.len() {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_single_element() {
    let input = vec![7.0];
    let kernel = vec![3.0];
    let out = tensor_convolution_1d::conv1d(&input, &kernel);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 21.0).abs() < 1e-9);
}

#[test]
fn test_basic() {
    // [1,2] * [3,4] = [3, 10, 8]  (full convolution: 1*3, 1*4+2*3, 2*4)
    let input = vec![1.0, 2.0];
    let kernel = vec![3.0, 4.0];
    let out = tensor_convolution_1d::conv1d(&input, &kernel);
    assert_eq!(out.len(), 3);
    assert!((out[0] - 3.0).abs() < 1e-9);
    assert!((out[1] - 10.0).abs() < 1e-9);
    assert!((out[2] - 8.0).abs() < 1e-9);
}
