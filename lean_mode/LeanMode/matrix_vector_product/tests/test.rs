#[test]
fn test_basic() {
    // 2x3 * 3x1 = 2x1
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // [[1,2,3],[4,5,6]]
    let x = vec![7.0, 8.0, 9.0];
    let y = matrix_vector_product::matvec(&a, &x, 2, 3);
    let expected = vec![1.0*7.0 + 2.0*8.0 + 3.0*9.0, 4.0*7.0 + 5.0*8.0 + 6.0*9.0];
    for i in 0..2 {
        assert!((y[i] - expected[i]).abs() < 1e-9);
    }
}

#[test]
fn test_identity() {
    // 2x2 identity * [3, 5]
    let a = vec![1.0, 0.0, 0.0, 1.0];
    let x = vec![3.0, 5.0];
    let y = matrix_vector_product::matvec(&a, &x, 2, 2);
    assert!((y[0] - 3.0).abs() < 1e-9);
    assert!((y[1] - 5.0).abs() < 1e-9);
}

#[test]
fn test_1x1() {
    let y = matrix_vector_product::matvec(&[2.0], &[3.0], 1, 1);
    assert!((y[0] - 6.0).abs() < 1e-9);
}

#[test]
fn test_zero() {
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let x = vec![0.0, 0.0];
    let y = matrix_vector_product::matvec(&a, &x, 2, 2);
    for v in y {
        assert!(v.abs() < 1e-9);
    }
}

#[test]
fn test_ones() {
    let a = vec![1.0; 6]; // 2x3 all ones
    let x = vec![1.0, 1.0, 1.0];
    let y = matrix_vector_product::matvec(&a, &x, 2, 3);
    assert!((y[0] - 3.0).abs() < 1e-9);
    assert!((y[1] - 3.0).abs() < 1e-9);
}
