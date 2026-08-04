use matrix_kronecker::kron;

#[test]
fn test_2x2_kron_2x2() {
    // A = [[1, 2],    B = [[5, 6],
    //      [3, 4]]         [7, 8]]
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![5.0, 6.0, 7.0, 8.0];
    let c = kron(&a, 2, 2, &b, 2, 2);

    let expected = vec![
        5.0, 6.0, 10.0, 12.0, 7.0, 8.0, 14.0, 16.0, 15.0, 18.0, 20.0, 24.0, 21.0, 24.0, 28.0, 32.0,
    ];
    let tol = 0.001;
    for i in 0..16 {
        let diff = c[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

#[test]
fn test_rectangular() {
    // A = 2x3, B = 2x2  →  output = 4x6
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let b = vec![7.0, 8.0, 9.0, 10.0];
    let c = kron(&a, 2, 3, &b, 2, 2);

    let expected = vec![
        7.0, 8.0, 14.0, 16.0, 21.0, 24.0, 9.0, 10.0, 18.0, 20.0, 27.0, 30.0, 28.0, 32.0, 35.0,
        40.0, 42.0, 48.0, 36.0, 40.0, 45.0, 50.0, 54.0, 60.0,
    ];
    let tol = 0.001;
    for i in 0..24 {
        let diff = c[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

#[test]
fn test_identity() {
    // A = I_2, B = [[2, 3], [4, 5]]
    let a = vec![1.0, 0.0, 0.0, 1.0];
    let b = vec![2.0, 3.0, 4.0, 5.0];
    let c = kron(&a, 2, 2, &b, 2, 2);

    let expected = vec![
        2.0, 3.0, 0.0, 0.0, 4.0, 5.0, 0.0, 0.0, 0.0, 0.0, 2.0, 3.0, 0.0, 0.0, 4.0, 5.0,
    ];
    let tol = 0.001;
    for i in 0..16 {
        let diff = c[i] - expected[i];
        assert!(diff.abs() <= tol);
    }
}

#[test]
fn test_1x1() {
    let a = vec![42.0];
    let b = vec![24.0];
    let c = kron(&a, 1, 1, &b, 1, 1);
    let tol = 0.001;
    let diff = c[0] - (42.0 * 24.0);
    assert!(diff.abs() <= tol);
}

#[test]
fn test_property() {
    // Verify K[i*M + k][j*R + l] = A[i][j] * B[k][l]
    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2x3
    let b = vec![10.0, 11.0, 12.0, 13.0]; // 2x2
    let c = kron(&a, 2, 3, &b, 2, 2);

    for i in 0..2usize {
        for j in 0..3usize {
            for k in 0..2usize {
                for l in 0..2usize {
                    let out_row = i * 2 + k;
                    let out_col = j * 2 + l;
                    let actual = c[out_row * 6 + out_col];
                    let expected = a[i * 3 + j] * b[k * 2 + l];
                    assert!((actual - expected).abs() < 1e-10);
                }
            }
        }
    }
}
