/// Test: 2×3 × 3×2 = 2×2
///
/// A = [[1, 2, 3],    B = [[7,  8],     C = [[58,  64],
///      [4, 5, 6]]         [9, 10],          [139, 154]]
///                         [11,12]]
#[test]
fn matmul_2x3_times_3x2() {
    let m = 2;
    let k = 3;
    let n = 2;

    let a = vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0];
    let b = vec![7.0, 9.0, 11.0, 8.0, 10.0, 12.0];
    let expected = vec![58.0, 139.0, 64.0, 154.0];

    let mut c = vec![0.0; m * n];
    matmul::matmul(&mut c, &a, &b, m, k, n);

    let tol = 0.001;
    for i in 0..m * n {
        let diff = c[i] - expected[i];
        assert!(
            diff.abs() <= tol,
            "c[{i}] = {}, expected {}",
            c[i],
            expected[i]
        );
    }
}
