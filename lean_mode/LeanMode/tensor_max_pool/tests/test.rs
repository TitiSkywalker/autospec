#[test]
fn test_empty() {
    let out = tensor_max_pool::max_pool2d(&[], 0, 0, 2, 2);
    assert!(out.is_empty());
    let out2 = tensor_max_pool::max_pool2d(&[1.0, 2.0], 1, 2, 0, 1);
    assert!(out2.is_empty());
}

#[test]
fn test_window_too_large() {
    let out = tensor_max_pool::max_pool2d(&[1.0, 2.0, 3.0, 4.0], 2, 2, 3, 2);
    assert!(out.is_empty());
}

#[test]
fn test_single() {
    let out = tensor_max_pool::max_pool2d(&[7.0], 1, 1, 1, 1);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 7.0).abs() < 1e-9);
}

#[test]
fn test_basic_2x2() {
    let input = vec![1.0, 2.0, 3.0, 4.0];
    let out = tensor_max_pool::max_pool2d(&input, 2, 2, 2, 2);
    assert_eq!(out.len(), 1);
    assert!((out[0] - 4.0).abs() < 1e-9);
}

#[test]
fn test_4x4_2x2() {
    let input = vec![
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0,
    ];
    let out = tensor_max_pool::max_pool2d(&input, 4, 4, 2, 2);
    assert_eq!(out.len(), 9);
    let expected = vec![6.0, 7.0, 8.0, 10.0, 11.0, 12.0, 14.0, 15.0, 16.0];
    for i in 0..9 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_negative() {
    let input = vec![-1.0, -2.0, -3.0, -5.0];
    let out = tensor_max_pool::max_pool2d(&input, 2, 2, 2, 2);
    assert_eq!(out.len(), 1);
    assert!((out[0] - (-1.0)).abs() < 1e-9);
}

#[test]
fn test_3x4_3x2() {
    // 3x4 input, 3x2 window -> 1x3 output
    let input = vec![
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
    ];
    let out = tensor_max_pool::max_pool2d(&input, 3, 4, 3, 2);
    assert_eq!(out.len(), 3);
    let expected = vec![10.0, 11.0, 12.0];
    for i in 0..3 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}