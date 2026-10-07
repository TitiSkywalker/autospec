//! n×m matrix rank via Gaussian elimination with partial pivoting.

/// Compute the rank of an n×m matrix stored in row-major order. Reduces the matrix to row echelon form with partial pivoting and returns the number of non-zero pivot rows.
pub fn rank(matrix: &[f64], nrows: usize, ncols: usize) -> usize {
    let mut a = matrix.to_vec();
    let mut rank: usize = 0;
    let mut col: usize = 0;

    while col < ncols && rank < nrows {
        // Partial pivoting: find the row with the largest |a[r][col]| among
        // rows r >= rank.
        let mut pivot_row = rank;
        let mut max_val = a[rank * ncols + col];
        if max_val < 0.0 {
            max_val = -max_val;
        }
        let mut r = rank + 1;
        while r < nrows {
            let mut v = a[r * ncols + col];
            if v < 0.0 {
                v = -v;
            }
            if v > max_val {
                max_val = v;
                pivot_row = r;
            }
            r += 1;
        }

        if max_val == 0.0 {
            // Column `col` is all zeros below row `rank` — skip it.
            col += 1;
        } else {
            // Swap the pivot row into position `rank`.
            if pivot_row != rank {
                let mut j = col;
                while j < ncols {
                    let tmp = a[rank * ncols + j];
                    a[rank * ncols + j] = a[pivot_row * ncols + j];
                    a[pivot_row * ncols + j] = tmp;
                    j += 1;
                }
            }

            // Eliminate column `col` in rows below `rank`.
            let pivot = a[rank * ncols + col];
            let mut r = rank + 1;
            while r < nrows {
                let factor = a[r * ncols + col] / pivot;
                let mut j = col + 1;
                while j < ncols {
                    a[r * ncols + j] = a[r * ncols + j] - factor * a[rank * ncols + j];
                    j += 1;
                }
                r += 1;
            }

            rank += 1;
            col += 1;
        }
    }

    rank
}
