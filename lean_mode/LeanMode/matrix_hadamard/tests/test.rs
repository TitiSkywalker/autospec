#[test]
fn test_basic() {
    // 2x2: [1,2; 3,4] ⊙ [5,6; 7,8] = [5,12; 21,32]
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![5.0, 6.0, 7.0, 8.0];
    let out = matrix_hadamard::hadamard(&a, &b, 2, 2);
    let expected = vec![5.0, 12.0, 21.0, 32.0];
    for i in 0..4 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_3x3() {
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let b = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let out = matrix_hadamard::hadamard(&a, &b, 3, 3);
    let expected = vec![1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0];
    for i in 0..9 {
        assert!((out[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_1x1() {
    let out = matrix_hadamard::hadamard(&[3.0], &[7.0], 1, 1);
    assert!((out[0] - 21.0).abs() < 1e-9);
}

#[test]
fn test_zero() {
    let a = vec![1.0; 6];
    let b = vec![0.0; 6];
    let out = matrix_hadamard::hadamard(&a, &b, 2, 3);
    for v in out {
        assert!(v.abs() < 1e-9);
    }
}

#[test]
fn test_identity() {
    // Element-wise with ones = identity
    let a = vec![3.0, 5.0, 7.0, 11.0];
    let b = vec![1.0, 1.0, 1.0, 1.0];
    let out = matrix_hadamard::hadamard(&a, &b, 2, 2);
    for i in 0..4 {
        assert!((out[i] - a[i]).abs() < 1e-9);
    }
}
