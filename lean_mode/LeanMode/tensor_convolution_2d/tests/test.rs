/// Test: identity kernel (center=1, rest=0) preserves input.
///
/// Input 3×3, kernel 3×3 with 1 at center.
#[test]
fn test_identity() {
    // 3x3 input: [1,2,3; 4,5,6; 7,8,9]
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    // 3x3 kernel: identity (1 at center)
    let kernel = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
    let out = tensor_convolution_2d::conv2d(&input, 3, 3, &kernel, 3, 3);
    // Output: 5x5, with input centered
    assert_eq!(out.len(), 25);
    // Center 3x3 should match input
    for i in 0..3 {
        for j in 0..3 {
            let out_idx = (i + 1) * 5 + (j + 1);
            let in_idx = i * 3 + j;
            assert!((out[out_idx] - input[in_idx]).abs() < 1e-9);
        }
    }
    // Corners should be zero
    assert!((out[0] - 0.0).abs() < 1e-9);
    assert!((out[4] - 0.0).abs() < 1e-9);
    assert!((out[20] - 0.0).abs() < 1e-9);
    assert!((out[24] - 0.0).abs() < 1e-9);
}

/// Test: 2x2 input, 2x2 kernel — basic correctness.
#[test]
fn test_basic() {
    // 2x2 input: [1,2; 3,4]
    let input = vec![1.0, 2.0, 3.0, 4.0];
    // 2x2 kernel: [5,6; 7,8]
    let kernel = vec![5.0, 6.0, 7.0, 8.0];
    let out = tensor_convolution_2d::conv2d(&input, 2, 2, &kernel, 2, 2);
    // Output: 3x3
    assert_eq!(out.len(), 9);
    // Manual calculation:
    // out[0,0] = input[0,0]*kernel[0,0] = 1*5 = 5
    // out[0,1] = input[0,0]*kernel[0,1] + input[0,1]*kernel[0,0] = 1*6 + 2*5 = 16
    // out[0,2] = input[0,1]*kernel[0,1] = 2*6 = 12
    // out[1,0] = input[0,0]*kernel[1,0] + input[1,0]*kernel[0,0] = 1*7 + 3*5 = 22
    // out[1,1] = input[0,0]*kernel[1,1] + input[0,1]*kernel[1,0]
    //           + input[1,0]*kernel[0,1] + input[1,1]*kernel[0,0]
    //           = 1*8 + 2*7 + 3*6 + 4*5 = 8 + 14 + 18 + 20 = 60
    // out[1,2] = input[0,1]*kernel[1,1] + input[1,1]*kernel[0,1] = 2*8 + 4*6 = 44
    // out[2,0] = input[1,0]*kernel[1,0] = 3*7 = 21
    // out[2,1] = input[1,0]*kernel[1,1] + input[1,1]*kernel[1,0] = 3*8 + 4*7 = 52
    // out[2,2] = input[1,1]*kernel[1,1] = 4*8 = 32
    let expected = vec![5.0, 16.0, 12.0, 22.0, 60.0, 40.0, 21.0, 52.0, 32.0];
    for i in 0..9 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

/// Test: 1x1 input, 1x1 kernel.
#[test]
fn test_single() {
    let out = tensor_convolution_2d::conv2d(&[7.0], 1, 1, &[3.0], 1, 1);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 21.0).abs() < 1e-9);
}

/// Test: 1x3 input, 1x3 kernel (behaves like 1D).
#[test]
fn test_1d_equivalent() {
    // 1x3 input
    let input = vec![1.0, 2.0, 3.0];
    // 1x3 kernel (same as 1D kernel [1, 2, 1])
    let kernel = vec![1.0, 2.0, 1.0];
    let out = tensor_convolution_2d::conv2d(&input, 1, 3, &kernel, 1, 3);
    // Output: 1x5
    assert_eq!(out.len(), 5);
    // Manual: [1, 1*2+2*1=4, 1*1+2*2+3*1=8, 2*1+3*2=8, 3*1=3]
    let expected = vec![1.0, 4.0, 8.0, 8.0, 3.0];
    for i in 0..5 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

/// Test: 2x2 input, 3x3 kernel.
#[test]
fn test_small_input_large_kernel() {
    // 2x2 input: [1,2; 3,4]
    let input = vec![1.0, 2.0, 3.0, 4.0];
    // 3x3 all-ones kernel
    let kernel = vec![1.0; 9];
    let out = tensor_convolution_2d::conv2d(&input, 2, 2, &kernel, 3, 3);
    // Output: 4x4 = 16 elements
    assert_eq!(out.len(), 16);
    // out[0,0] = 1*1 = 1
    assert!((out[0] - 1.0).abs() < 1e-9);
    // out[3,3] = 4*1 = 4
    assert!((out[15] - 4.0).abs() < 1e-9);
    // out[1,1] = 1+2+3+4 = 10
    assert!((out[5] - 10.0).abs() < 1e-9);
}

/// Test: empty dimension.
#[test]
fn test_empty() {
    let out = tensor_convolution_2d::conv2d(&[], 0, 0, &[1.0], 1, 1);
    assert!(out.is_empty());
    let out2 = tensor_convolution_2d::conv2d(&[1.0], 1, 1, &[], 0, 0);
    assert!(out2.is_empty());
}
