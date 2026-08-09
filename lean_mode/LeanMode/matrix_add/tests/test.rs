#[test]
fn test_basic() {
    // 2x2: [1,2; 3,4] + [5,6; 7,8] = [6,8; 10,12]
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![5.0, 6.0, 7.0, 8.0];
    let out = matrix_add::matadd(&a, &b, 2, 2);
    let expected = vec![6.0, 8.0, 10.0, 12.0];
    for i in 0..4 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_3x3() {
    let a = vec![1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0, 5.0];
    let b = vec![0.0, 1.0, 0.0, 2.0, 0.0, 3.0, 0.0, 4.0, 0.0];
    let out = matrix_add::matadd(&a, &b, 3, 3);
    let expected = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0];
    for i in 0..9 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_1x1() {
    let out = matrix_add::matadd(&[3.0], &[7.0], 1, 1);
    assert!((out[0] - 10.0).abs() < 1e-9);
}

#[test]
fn test_zero() {
    // 0 + 0 = 0
    let a = vec![0.0; 4];
    let b = vec![0.0; 4];
    let out = matrix_add::matadd(&a, &b, 2, 2);
    for v in out {
        assert!(v.abs() < 1e-9);
    }
}

#[test]
fn test_negative() {
    let a = vec![1.0, -2.0, 3.0, -4.0];
    let b = vec![-1.0, 2.0, -3.0, 4.0];
    let out = matrix_add::matadd(&a, &b, 2, 2);
    for v in out {
        assert!(v.abs() < 1e-9);
    }
}
