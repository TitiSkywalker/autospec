//! Bunch–Kaufman LBLᵀ factorization of a symmetric (possibly indefinite) matrix.
//!
//! Extracted from nalgebra's `LBLT::new` + `l_permuted` + `d` (the
//! block-diagonal-pivoting method) and refined for Aeneas compatibility (while
//! loops, index-based access, no iterators/closures).
//!
//! Decomposes a real symmetric n×n matrix A into A = L · B · Lᵀ, where `L` is a
//! (permuted) unit-lower-triangular factor and `B` is block diagonal with 1×1
//! and 2×2 blocks.  Unlike LDLᵀ, this also works for indefinite matrices (via
//! the 2×2 pivot blocks).
//!
//! All matrices are stored in **column-major** order: element (row, col) is at
//! index `row + col * n`.  Only the lower triangle (including the diagonal) of
//! the input is read.

/// Result of the LBLᵀ factorization.
pub struct Lblt {
    /// Permuted unit-lower factor L (n×n, column-major; in general *not*
    /// lower-triangular in the original basis).
    pub l: Vec<f64>,
    /// Block-diagonal factor B (n×n, column-major; 1×1 and 2×2 blocks).
    pub d: Vec<f64>,
}

/// Absolute value, hand-expanded (no `.abs()` method call).
fn abs_val(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// Swap columns `a` and `b` of an n×n column-major matrix, consuming and
/// returning it.
fn swap_columns(mut m: Vec<f64>, n: usize, a: usize, b: usize) -> Vec<f64> {
    let mut r: usize = 0;
    while r < n {
        let ia = r + a * n;
        let ib = r + b * n;
        let tmp = m[ia];
        m[ia] = m[ib];
        m[ib] = tmp;
        r += 1;
    }
    m
}

/// Reconstruct the permuted unit-lower factor L = Pᵀ·L̂ from the stored
/// multipliers and pivot sequence.
fn build_l(m: &[f64], piv_index: &[usize], piv_size: &[usize], n: usize) -> Vec<f64> {
    let mut l = vec![0.0; n * n];
    let mut i: usize = 0;
    while i < n {
        l[i + i * n] = 1.0;
        i += 1;
    }

    let mut k: usize = 0;
    while k < n {
        let pivot_index = piv_index[k];
        let block_size = piv_size[k];

        if block_size == 1 {
            l = swap_columns(l, n, k, pivot_index);

            let mut row: usize = 0;
            while row < n {
                let mut acc: f64 = 0.0;
                let mut i: usize = k + 1;
                while i < n {
                    acc = acc + l[row + i * n] * m[i + k * n];
                    i += 1;
                }
                let idx = row + k * n;
                let new_val = l[idx] + acc;
                l[idx] = new_val;
                row += 1;
            }

            k += 1;
        } else {
            l = swap_columns(l, n, k + 1, pivot_index);

            let mut row: usize = 0;
            while row < n {
                let mut acc1: f64 = 0.0;
                let mut acc2: f64 = 0.0;
                let mut i: usize = k + 2;
                while i < n {
                    acc1 = acc1 + l[row + i * n] * m[i + k * n];
                    acc2 = acc2 + l[row + i * n] * m[i + (k + 1) * n];
                    i += 1;
                }
                let idx1 = row + k * n;
                let idx2 = row + (k + 1) * n;
                let new1 = l[idx1] + acc1;
                let new2 = l[idx2] + acc2;
                l[idx1] = new1;
                l[idx2] = new2;
                row += 1;
            }

            k += 2;
        }
    }

    l
}

/// Reconstruct the block-diagonal factor B from the stored diagonal blocks.
fn build_d(m: &[f64], piv_size: &[usize], n: usize) -> Vec<f64> {
    let mut d = vec![0.0; n * n];

    let mut k: usize = 0;
    while k < n {
        d[k + k * n] = m[k + k * n];
        if piv_size[k] == 2 {
            d[(k + 1) + k * n] = m[(k + 1) + k * n];
            d[k + (k + 1) * n] = m[(k + 1) + k * n];
            d[(k + 1) + (k + 1) * n] = m[(k + 1) + (k + 1) * n];
            k += 1;
        }
        k += 1;
    }

    d
}

/// Compute the Bunch–Kaufman LBLᵀ factorization of a real symmetric n×n matrix
/// (column-major).
///
/// Returns L (n×n, permuted unit-lower) and B (n×n, block diagonal) such that
/// A = L · B · Lᵀ.
///
/// * `matrix` — flattened n×n symmetric matrix in column-major order (only the
///   lower triangle is read)
/// * `n` — dimension
pub fn lblt_decomposition(matrix: &[f64], n: usize) -> Lblt {
    if n == 0 {
        return Lblt {
            l: Vec::new(),
            d: Vec::new(),
        };
    }

    let mut m = matrix.to_vec();
    let mut piv_index: Vec<usize> = vec![0; n];
    let mut piv_size: Vec<usize> = vec![0; n];

    // Bunch–Kaufman pivot threshold: (1 + sqrt(17)) / 8.
    let alpha: f64 = 0.6403882032022076;

    let mut k: usize = 0;
    while k < n {
        let mut block_size: usize = 1;

        let diag_abs = abs_val(m[k + k * n]);

        // Largest off-diagonal entry of the active part of column k.
        let mut imax: usize = 0;
        let mut colmax: f64 = 0.0;
        let has_off = k + 1 < n;
        if has_off {
            imax = k + 1;
            colmax = abs_val(m[(k + 1) + k * n]);
            let mut i: usize = k + 2;
            while i < n {
                let mag = abs_val(m[i + k * n]);
                let bigger = mag > colmax;
                if bigger {
                    imax = i;
                    colmax = mag;
                }
                i += 1;
            }
        }

        let max_dc = if diag_abs > colmax { diag_abs } else { colmax };
        let is_zero_col = max_dc == 0.0;
        if is_zero_col {
            piv_index[k] = k;
            piv_size[k] = 1;
            k += 1;
        } else {
            let pivot_index: usize;
            let diag_lt = diag_abs < alpha * colmax;
            if diag_lt {
                let mut rowmax: f64 = 0.0;
                let mut j: usize = k;
                while j < imax {
                    let rv = abs_val(m[imax + j * n]);
                    let bigger = rv > rowmax;
                    if bigger {
                        rowmax = rv;
                    }
                    j += 1;
                }
                let mut j: usize = imax + 1;
                while j < n {
                    let rv = abs_val(m[j + imax * n]);
                    let bigger = rv > rowmax;
                    if bigger {
                        rowmax = rv;
                    }
                    j += 1;
                }

                let ratio = colmax / rowmax;
                let keep_1x1 = diag_abs >= alpha * colmax * ratio;
                if keep_1x1 {
                    pivot_index = k;
                } else {
                    pivot_index = imax;
                    let cand_small = abs_val(m[imax + imax * n]) < alpha * rowmax;
                    if cand_small {
                        block_size = 2;
                    }
                }
            } else {
                pivot_index = k;
            }

            let pivot_target = k + block_size - 1;

            if pivot_index != pivot_target {
                // Symmetric two-sided interchange.
                let mut i: usize = pivot_index + 1;
                while i < n {
                    let ia = i + pivot_target * n;
                    let ib = i + pivot_index * n;
                    let tmp = m[ia];
                    m[ia] = m[ib];
                    m[ib] = tmp;
                    i += 1;
                }
                let mut j: usize = pivot_target + 1;
                while j < pivot_index {
                    let ja = j + pivot_target * n;
                    let jb = pivot_index + j * n;
                    let tmp = m[ja];
                    m[ja] = m[jb];
                    m[jb] = tmp;
                    j += 1;
                }
                let da = pivot_target + pivot_target * n;
                let db = pivot_index + pivot_index * n;
                let tmp = m[da];
                m[da] = m[db];
                m[db] = tmp;

                let edge = k + 1 == pivot_target;
                if edge {
                    let ea = (k + 1) + k * n;
                    let eb = pivot_index + k * n;
                    let tmp = m[ea];
                    m[ea] = m[eb];
                    m[eb] = tmp;
                }
            }

            if block_size == 1 {
                // 1×1 pivot: rank-1 update + normalize the multiplier column.
                let has_trail = k + 1 < n;
                if has_trail {
                    let inv_diag = 1.0 / m[k + k * n];

                    let mut j: usize = k + 1;
                    while j < n {
                        let jk = m[j + k * n];
                        let mut i: usize = j;
                        while i < n {
                            let idx = i + j * n;
                            let new_val = m[idx] - inv_diag * m[i + k * n] * jk;
                            m[idx] = new_val;
                            i += 1;
                        }
                        j += 1;
                    }

                    let mut i: usize = k + 1;
                    while i < n {
                        let idx = i + k * n;
                        let new_val = m[idx] * inv_diag;
                        m[idx] = new_val;
                        i += 1;
                    }
                }

                piv_index[k] = pivot_index;
                piv_size[k] = 1;
            } else {
                // 2×2 pivot: rank-2 update.
                let has_trail = k + 2 < n;
                if has_trail {
                    let dval = abs_val(m[(k + 1) + k * n]);
                    let d11 = m[(k + 1) + (k + 1) * n] / dval;
                    let d22 = m[k + k * n] / dval;
                    let d21 = m[(k + 1) + k * n] / dval;
                    let scale = 1.0 / (dval * (d11 * d22 - 1.0));

                    let mut j: usize = k + 2;
                    while j < n {
                        let work1 = (m[j + k * n] * d11 - m[j + (k + 1) * n] * d21) * scale;
                        let work2 = (m[j + (k + 1) * n] * d22 - m[j + k * n] * d21) * scale;
                        let mut i: usize = j;
                        while i < n {
                            let idx = i + j * n;
                            let new_val = m[idx]
                                - m[i + k * n] * work1
                                - m[i + (k + 1) * n] * work2;
                            m[idx] = new_val;
                            i += 1;
                        }
                        m[j + k * n] = work1;
                        m[j + (k + 1) * n] = work2;
                        j += 1;
                    }
                }

                piv_index[k] = pivot_index;
                piv_size[k] = 2;
                piv_index[k + 1] = pivot_index;
                piv_size[k + 1] = 2;
            }

            k += block_size;
        }
    }

    let l = build_l(&m, &piv_index, &piv_size, n);
    let d = build_d(&m, &piv_size, n);

    Lblt { l, d }
}
