//! n×n matrix determinant via Gaussian elimination with partial pivoting.

/// Compute the determinant of an n×n matrix stored in row-major order.
///
/// Returns 0.0 for singular matrices.  Uses Gaussian elimination with
/// partial pivoting for numerical stability.
pub fn det(matrix: &[f64], n: usize) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let mut a = matrix.to_vec();
    let mut sign = 1.0;

    let mut k: usize = 0;
    while k < n {
        // Partial pivoting: find the row with the largest pivot in column k.
        let mut max_row = k;
        let mut max_val = a[k * n + k];
        if max_val < 0.0 {
            max_val = -max_val;
        }
        let mut i = k + 1;
        while i < n {
            let mut v = a[i * n + k];
            if v < 0.0 {
                v = -v;
            }
            if v > max_val {
                max_val = v;
                max_row = i;
            }
            i += 1;
        }

        // Singular matrix — determinant is zero.
        if max_val == 0.0 {
            return 0.0;
        }

        // Swap rows if needed.
        if max_row != k {
            let mut j = k;
            while j < n {
                let tmp = a[k * n + j];
                a[k * n + j] = a[max_row * n + j];
                a[max_row * n + j] = tmp;
                j += 1;
            }
            sign = -sign;
        }

        // Eliminate entries below the pivot.
        let pivot = a[k * n + k];
        let mut i = k + 1;
        while i < n {
            let factor = a[i * n + k] / pivot;
            let mut j = k + 1;
            while j < n {
                a[i * n + j] = a[i * n + j] - factor * a[k * n + j];
                j += 1;
            }
            i += 1;
        }

        k += 1;
    }

    let mut det = sign;
    let mut i: usize = 0;
    while i < n {
        det = det * a[i * n + i];
        i += 1;
    }
    det
}
