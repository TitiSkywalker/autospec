//! Simple f64 matrix multiplication (column-major storage).

/// Multiply two column-major matrices.
pub fn matmul(c: &mut [f64], a: &[f64], b: &[f64], m: usize, k: usize, n: usize) {
    for col_b in 0..n {
        for row in 0..m {
            for col_a in 0..k {
                let c_idx = row + col_b * m;
                let a_idx = row + col_a * m;
                let b_idx = col_a + col_b * k;
                c[c_idx] = c[c_idx] + a[a_idx] * b[b_idx];
            }
        }
    }
}
