//! Kronecker product of two matrices (row-major storage).

/// Kronecker product of a L×N matrix A and a M×R matrix B.
///
/// Returns a (L·M) × (N·R) matrix K in row-major order,
/// where `K[i·M + k][j·R + l] = A[i][j] · B[k][l]`.
pub fn kron(
    a: &[f64],
    a_rows: usize,
    a_cols: usize,
    b: &[f64],
    b_rows: usize,
    b_cols: usize,
) -> Vec<f64> {
    let out_rows = a_rows * b_rows;
    let out_cols = a_cols * b_cols;
    let mut out = vec![0.0; out_rows * out_cols];

    for i in 0..a_rows {
        for j in 0..a_cols {
            let a_val = a[i * a_cols + j];
            for k in 0..b_rows {
                for l in 0..b_cols {
                    let b_val = b[k * b_cols + l];
                    let out_row = i * b_rows + k;
                    let out_col = j * b_cols + l;
                    out[out_row * out_cols + out_col] = a_val * b_val;
                }
            }
        }
    }
    out
}
