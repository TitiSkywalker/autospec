//! Symmetric eigendecomposition via cyclic Jacobi rotations.
//!
//! Decomposes a real symmetric n×n matrix A into A = V · Λ · Vᵀ, where V is an
//! orthogonal matrix whose columns are eigenvectors, and Λ = diag(values) holds
//! the corresponding (real) eigenvalues on its diagonal.
//!
//! All matrices are stored in **row-major** order: element (row, col) is at
//! index `row * n + col`. Column `j` of `vectors` is the eigenvector associated
//! with eigenvalue `values[j]`.

/// Result of the symmetric eigendecomposition.
pub struct Eigen {
    /// Eigenvalues λ₁ … λ_n (real; not necessarily sorted).
    pub values: Vec<f64>,
    /// Eigenvectors as an n×n matrix in row-major order: column `j` is the
    /// (unit) eigenvector for `values[j]`.
    pub vectors: Vec<f64>,
}

/// Apply one Jacobi rotation to the off-diagonal entries at row `k` of the
/// symmetric matrix `a` (rows `p` and `q` are excluded by the caller).
///
/// Updates `a[k][p]`, `a[k][q]` and, by symmetry, `a[p][k]`, `a[q][k]`.
fn update_pair(
    mut a: Vec<f64>,
    n: usize,
    p: usize,
    q: usize,
    c: f64,
    s: f64,
    k: usize,
) -> Vec<f64> {
    let a_kp = a[k * n + p];
    let a_kq = a[k * n + q];
    let new_kp = c * a_kp + s * a_kq;
    let new_kq = c * a_kq - s * a_kp;
    a[k * n + p] = new_kp;
    a[k * n + q] = new_kq;
    a[p * n + k] = new_kp;
    a[q * n + k] = new_kq;
    a
}

/// Compute the eigendecomposition of a real symmetric n×n matrix (row-major).
///
/// Uses the classical (two-sided) Jacobi eigenvalue algorithm: successive
/// orthogonal rotations zero out off-diagonal entries until the working matrix
/// is diagonal.  The diagonal then holds the eigenvalues and the accumulated
/// rotations give the eigenvectors.
///
/// * `matrix` — flattened n×n symmetric matrix in row-major order
/// * `n` — dimension
/// * `eps` — convergence threshold on the largest off-diagonal magnitude
/// * `max_sweeps` — maximum number of Jacobi sweeps
///
/// # Precondition
///
/// `matrix` must be symmetric (`matrix[i][j] == matrix[j][i]` for all `i`, `j`).
/// The Jacobi method only diagonalizes symmetric matrices, whose eigenvalues are
/// all real.  This precondition is stated explicitly in `Spec.lean` as the
/// `IsSymmetric` hypothesis; it is **not** checked here (no runtime `assert!`),
/// so a non-symmetric input silently produces a result that is not a valid
/// eigendecomposition.
pub fn symmetric_eigen(
    matrix: &[f64],
    n: usize,
    eps: f64,
    max_sweeps: usize,
) -> Eigen {
    if n == 0 {
        return Eigen {
            values: Vec::new(),
            vectors: Vec::new(),
        };
    }

    // Precondition (documented above): `matrix` is symmetric.  Only the strictly
    // upper triangle is touched by the off-diagonal update below, and it is kept
    // mirrored to the lower triangle via `update_pair`, so the working copy
    // stays symmetric throughout.

    // Working copy of A (becomes diagonal as rotations are applied).
    let mut a = matrix.to_vec();

    // Accumulated eigenvectors V, initialized to the identity.
    let mut v = vec![0.0; n * n];
    let mut i: usize = 0;
    while i < n {
        v[i * n + i] = 1.0;
        i += 1;
    }

    let mut sweep: usize = 0;
    let mut done = false;
    while !done && sweep < max_sweeps {
        let mut max_off: f64 = 0.0;

        let mut p: usize = 0;
        while p < n {
            let mut q: usize = p + 1;
            while q < n {
                let a_pq = a[p * n + q];
                let a_pq_neg = a_pq < 0.0;
                let abs_pq = if a_pq_neg { -a_pq } else { a_pq };
                let off_bigger = abs_pq > max_off;
                if off_bigger {
                    max_off = abs_pq;
                }

                // Only rotate entries above the convergence threshold.
                let rotate = abs_pq > eps;
                if rotate {
                    let a_pp = a[p * n + p];
                    let a_qq = a[q * n + q];
                    let two: f64 = 2.0;

                    // τ = (a_pp − a_qq) / (2·a_pq); t is the smaller root of
                    // t² + 2·τ·t − 1 = 0 (tan of the rotation angle θ).
                    let tau = (a_pp - a_qq) / (two * a_pq);
                    let tau_neg = tau < 0.0;
                    let abs_tau = if tau_neg { -tau } else { tau };
                    let sqrt_term = (1.0 + tau * tau).sqrt();
                    let denom = abs_tau + sqrt_term;
                    let t_sign = if tau_neg { -1.0 } else { 1.0 };
                    let t = t_sign / denom;
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;

                    // Update the diagonal and zero out the (p, q) block.
                    a[p * n + p] = a_pp + t * a_pq;
                    a[q * n + q] = a_qq - t * a_pq;
                    a[p * n + q] = 0.0;
                    a[q * n + p] = 0.0;

                    // Update columns p and q for every row k ∉ {p, q}, plus their
                    // symmetric counterparts. Three disjoint ranges avoid p and q.
                    let mut k: usize = 0;
                    while k < p {
                        a = update_pair(a, n, p, q, c, s, k);
                        k += 1;
                    }
                    let mut k: usize = p + 1;
                    while k < q {
                        a = update_pair(a, n, p, q, c, s, k);
                        k += 1;
                    }
                    let mut k: usize = q + 1;
                    while k < n {
                        a = update_pair(a, n, p, q, c, s, k);
                        k += 1;
                    }

                    // Accumulate the rotation into V: V ← V·G, so columns p and
                    // q of V transform exactly like columns p and q of A.
                    let mut k: usize = 0;
                    while k < n {
                        let v_kp = v[k * n + p];
                        let v_kq = v[k * n + q];
                        v[k * n + p] = c * v_kp + s * v_kq;
                        v[k * n + q] = c * v_kq - s * v_kp;
                        k += 1;
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

    // Extract the eigenvalues (the diagonal of the now-diagonal working matrix).
    let mut values = vec![0.0; n];
    let mut i: usize = 0;
    while i < n {
        values[i] = a[i * n + i];
        i += 1;
    }

    Eigen { values, vectors: v }
}
