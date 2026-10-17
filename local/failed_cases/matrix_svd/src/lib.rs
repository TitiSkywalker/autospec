//! Singular Value Decomposition via one-sided Jacobi rotations.
//!
//! Decomposes an m×n matrix A into A = U · Σ · Vᵀ, where U (m×k) and V (n×k)
//! have orthonormal columns and Σ (k×k) is diagonal with non-negative entries,
//! with k = min(m, n).
//!
//! All matrices are stored in **row-major** order: element (row, col) is at
//! index `row * ncols + col` (for U: `row * k + col`; for Vᵀ: `row * n + col`).

/// Result of the singular value decomposition.
pub struct Svd {
    /// U factor: m×k matrix with orthonormal columns (row-major).
    pub u: Vec<f64>,
    /// Singular values σ₁ … σ_k (non-negative, not necessarily sorted).
    pub singular_values: Vec<f64>,
    /// Vᵀ factor: k×n matrix (row-major, k rows, n columns).
    pub v_t: Vec<f64>,
}

/// Transpose a row-major matrix with `rows` rows and `cols` columns.
///
/// The result has `cols` rows and `rows` columns: element (i, j) of `b` is
/// element (j, i) of `a`.
fn transpose(a: &[f64], rows: usize, cols: usize) -> Vec<f64> {
    let mut b = vec![0.0; rows * cols];
    let mut j: usize = 0;
    while j < cols {
        let mut i: usize = 0;
        while i < rows {
            b[j * rows + i] = a[i * cols + j];
            i += 1;
        }
        j += 1;
    }
    b
}

/// One-sided Jacobi SVD of a tall (or square) matrix: `rows ≥ cols`.
///
/// Returns `(u, s, v)` with `A = U · Σ · Vᵀ`, where `u` is rows×cols with
/// orthonormal columns, `s` holds the singular values (column norms), and
/// `v` is the cols×cols accumulated rotation (so `Vᵀ = transpose(v)`).
fn jacobi_svd_tall(
    a: &[f64],
    rows: usize,
    cols: usize,
    eps: f64,
    max_sweeps: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut work = a.to_vec();

    // Accumulated rotation V, initialized to the identity.
    let mut v = vec![0.0; cols * cols];
    let mut i: usize = 0;
    while i < cols {
        v[i * cols + i] = 1.0;
        i += 1;
    }

    let mut sweep: usize = 0;
    let mut done = false;
    while !done && sweep < max_sweeps {
        let mut max_off: f64 = 0.0;
        let mut p: usize = 0;
        while p < cols {
            let mut q: usize = p + 1;
            while q < cols {
                // α = ‖col_p‖², β = ‖col_q‖², γ = col_p · col_q
                let mut alpha: f64 = 0.0;
                let mut beta: f64 = 0.0;
                let mut gamma: f64 = 0.0;
                let mut r: usize = 0;
                while r < rows {
                    let x = work[r * cols + p];
                    let y = work[r * cols + q];
                    alpha = alpha + x * x;
                    beta = beta + y * y;
                    gamma = gamma + x * y;
                    r += 1;
                }

                let gamma_nonzero = gamma != 0.0;
                let prod = alpha * beta;
                let prod_pos = prod > 0.0;
                let mut rel: f64 = 0.0;
                let mut rotate = false;
                if gamma_nonzero && prod_pos {
                    let denom = prod.sqrt();
                    let abs_gamma = if gamma < 0.0 { -gamma } else { gamma };
                    rel = abs_gamma / denom;
                    rotate = rel > eps;
                }
                let rel_bigger = rel > max_off;
                if rel_bigger {
                    max_off = rel;
                }

                if rotate {
                    // ζ = (β − α) / (2γ), t = sign(ζ) / (|ζ| + √(1 + ζ²))
                    let two: f64 = 2.0;
                    let zeta = (beta - alpha) / (two * gamma);
                    let zeta_neg = zeta < 0.0;
                    let abs_zeta = if zeta_neg { -zeta } else { zeta };
                    let sign_zeta = if zeta_neg { -1.0 } else { 1.0 };
                    let sqrt_term = (1.0 + zeta * zeta).sqrt();
                    let t = sign_zeta / (abs_zeta + sqrt_term);
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = c * t;

                    // A ← A · G, V ← V · G (G acts on columns p, q)
                    let mut r: usize = 0;
                    while r < rows {
                        let x = work[r * cols + p];
                        let y = work[r * cols + q];
                        work[r * cols + p] = c * x - s * y;
                        work[r * cols + q] = s * x + c * y;
                        r += 1;
                    }
                    let mut r: usize = 0;
                    while r < cols {
                        let x = v[r * cols + p];
                        let y = v[r * cols + q];
                        v[r * cols + p] = c * x - s * y;
                        v[r * cols + q] = s * x + c * y;
                        r += 1;
                    }
                }

                q += 1;
            }
            p += 1;
        }
        sweep += 1;
        let converged = max_off <= eps;
        if converged {
            done = true;
        }
    }

    // Singular values are the column norms; U normalizes each column.
    let mut s = vec![0.0; cols];
    let mut u = vec![0.0; rows * cols];
    let mut j: usize = 0;
    while j < cols {
        let mut norm: f64 = 0.0;
        let mut r: usize = 0;
        while r < rows {
            norm = norm + work[r * cols + j] * work[r * cols + j];
            r += 1;
        }
        let sigma = norm.sqrt();
        s[j] = sigma;
        let sigma_pos = sigma > 0.0;
        if sigma_pos {
            let mut r: usize = 0;
            while r < rows {
                u[r * cols + j] = work[r * cols + j] / sigma;
                r += 1;
            }
        } else {
            let mut r: usize = 0;
            while r < rows {
                u[r * cols + j] = 0.0;
                r += 1;
            }
        }
        j += 1;
    }

    (u, s, v)
}

/// Compute the singular value decomposition of an m×n matrix (row-major).
///
/// Returns `Svd { u, singular_values, v_t }` with A = U · Σ · Vᵀ, where
/// U is m×k with orthonormal columns, Vᵀ is k×n with orthonormal rows, and
/// Σ = diag(singular_values) has non-negative entries, k = min(m, n).
pub fn svd(
    matrix: &[f64],
    nrows: usize,
    ncols: usize,
    eps: f64,
    max_sweeps: usize,
) -> Svd {
    let k = if nrows < ncols { nrows } else { ncols };
    let k_zero = k == 0;
    if k_zero {
        return Svd {
            u: Vec::new(),
            singular_values: Vec::new(),
            v_t: Vec::new(),
        };
    }

    let tall = nrows >= ncols;
    if tall {
        let (u, s, v) = jacobi_svd_tall(matrix, nrows, ncols, eps, max_sweeps);
        let v_t = transpose(&v, ncols, ncols);
        Svd {
            u,
            singular_values: s,
            v_t,
        }
    } else {
        // m < n: work on Aᵀ (n×m, tall) and swap the factors.
        let at = transpose(matrix, nrows, ncols);
        let (u_b, s, v_b) = jacobi_svd_tall(&at, ncols, nrows, eps, max_sweeps);
        // A = V_B · Σ · U_Bᵀ, so U = V_B and Vᵀ = U_Bᵀ.
        let v_t = transpose(&u_b, ncols, nrows);
        Svd {
            u: v_b,
            singular_values: s,
            v_t,
        }
    }
}
