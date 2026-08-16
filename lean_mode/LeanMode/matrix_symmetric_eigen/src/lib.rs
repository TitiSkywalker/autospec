//! Symmetric eigendecomposition via Householder tridiagonalization + implicit
//! QL iteration (Wilkinson shifts, Givens rotations).
//!
//! Extracted from nalgebra's `SymmetricEigen::do_decompose` (which calls
//! `SymmetricTridiagonal::new` and the implicit QL sweep) and refined for
//! Aeneas compatibility (while loops, index-based access, no iterators/closures,
//! no `break`/`continue`).
//!
//! Decomposes a real symmetric n×n matrix A into A = V · Λ · Vᵀ, where V is
//! orthogonal (its columns are eigenvectors) and Λ = diag(eigenvalues) holds the
//! real eigenvalues on its diagonal.
//!
//! All matrices are stored in **column-major** order: element (row, col) is at
//! index `row + col * n`. Column `j` of `eigenvectors` is the (unit)
//! eigenvector associated with `eigenvalues[j]`.

/// Result of the symmetric eigendecomposition.
pub struct SymmetricEigen {
    /// Eigenvalues λ₁ … λ_n (real; not necessarily sorted).
    pub eigenvalues: Vec<f64>,
    /// Eigenvectors as an n×n matrix in column-major order: column `j` is the
    /// (unit) eigenvector for `eigenvalues[j]`.
    pub eigenvectors: Vec<f64>,
}

/// Absolute value, hand-expanded (no `.abs()` method call).
fn abs_val(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// Compute the Householder reflection axis for a vector, in place.
///
/// Returns `(signed_norm, not_zero)` where `signed_norm` is the signed Euclidean
/// norm of the original vector; the vector is normalized to unit length.
fn reflection_axis(axis: &mut Vec<f64>, len: usize) -> (f64, bool) {
    if len == 0 {
        return (0.0, false);
    }

    let mut sq_norm: f64 = 0.0;
    let mut i: usize = 0;
    while i < len {
        sq_norm = sq_norm + axis[i] * axis[i];
        i += 1;
    }
    let norm = sq_norm.sqrt();
    if norm == 0.0 {
        return (0.0, false);
    }

    let first = axis[0];
    let sign = if first < 0.0 { -1.0 } else { 1.0 };
    let modulus = if first < 0.0 { -first } else { first };
    let signed_norm = sign * norm;

    let factor = (sq_norm + modulus * norm) * 2.0;
    if factor == 0.0 {
        return (signed_norm, false);
    }

    axis[0] = first + signed_norm;

    let factor_sqrt = factor.sqrt();
    let mut i: usize = 0;
    while i < len {
        axis[i] = axis[i] / factor_sqrt;
        i += 1;
    }

    (-signed_norm, true)
}

/// Reconstruct the orthogonal factor Q (n×n, column-major) by applying the
/// Householder reflections to the identity in reverse order.
fn assemble_q(uv: &[f64], signs: &[f64], n: usize) -> Vec<f64> {
    let mut q = vec![0.0; n * n];
    let mut i: usize = 0;
    while i < n {
        q[i + i * n] = 1.0;
        i += 1;
    }

    let mut ii: usize = n - 1;
    while ii > 0 {
        ii -= 1;
        let i = ii;
        let m = n - i - 1;

        let sign = if signs[i] < 0.0 { -1.0 } else { 1.0 };
        let two = -2.0 * sign;

        let mut col: usize = i;
        while col < n {
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < m {
                dot = dot + uv[(i + 1 + t) + i * n] * q[(i + 1 + t) + col * n];
                t += 1;
            }
            let factor = two * dot;
            let mut t: usize = 0;
            while t < m {
                let idx = (i + 1 + t) + col * n;
                let new_val = sign * q[idx] + factor * uv[(i + 1 + t) + i * n];
                q[idx] = new_val;
                t += 1;
            }
            col += 1;
        }
    }

    q
}

/// Reduce a symmetric n×n matrix to symmetric tridiagonal form via Householder
/// reflections.
///
/// Returns the compact form `tri` (Householder vectors in column `i`, rows
/// `i+1..n`; diagonal entries are the tridiagonal diagonal) and the signed
/// norms `off_diagonal` (length n-1).
fn tridiagonalize(mut tri: Vec<f64>, n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut off_diagonal = vec![0.0; n - 1];

    let mut i: usize = 0;
    while i < n - 1 {
        let m = n - i - 1;

        // Extract axis = column i, rows i+1..n.
        let mut axis = vec![0.0; m];
        let mut t: usize = 0;
        while t < m {
            axis[t] = tri[(i + 1 + t) + i * n];
            t += 1;
        }

        let (norm, not_zero) = reflection_axis(&mut axis, m);
        off_diagonal[i] = norm;

        // Store the (unit) Householder vector back into column i.
        let mut t: usize = 0;
        while t < m {
            tri[(i + 1 + t) + i * n] = axis[t];
            t += 1;
        }

        if not_zero {
            // p = 2 · M · axis, where M is the trailing submatrix (rows/cols i+1..n).
            let mut p = vec![0.0; m];
            let mut r: usize = 0;
            while r < m {
                let mut sum: f64 = 0.0;
                let mut t: usize = 0;
                while t < m {
                    sum = sum + tri[(i + 1 + r) + (i + 1 + t) * n] * axis[t];
                    t += 1;
                }
                p[r] = 2.0 * sum;
                r += 1;
            }

            // dot = axis · p.
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < m {
                dot = dot + axis[t] * p[t];
                t += 1;
            }

            // Two-sided update M ← H · M · H, expanded as three rank-1 updates:
            //   M ← M - p·axisᵀ - axis·pᵀ + 2·dot·axis·axisᵀ.
            let mut r: usize = 0;
            while r < m {
                let mut c: usize = 0;
                while c < m {
                    let idx = (i + 1 + r) + (i + 1 + c) * n;
                    let update = -p[r] * axis[c] - axis[r] * p[c]
                        + 2.0 * dot * axis[r] * axis[c];
                    let new_val = tri[idx] + update;
                    tri[idx] = new_val;
                    c += 1;
                }
                r += 1;
            }
        }

        i += 1;
    }

    (tri, off_diagonal)
}

/// Wilkinson shift: the eigenvalue of the trailing 2×2 block
/// `[[tmm, tmn], [tmn, tnn]]` closest to `tnn`.
fn wilkinson_shift(tmm: f64, tnn: f64, tmn: f64) -> f64 {
    let sq_tmn = tmn * tmn;
    if sq_tmn != 0.0 {
        let d = (tmm - tnn) * 0.5;
        let d_sign = if d < 0.0 { -1.0 } else { 1.0 };
        let sqrt_term = (d * d + sq_tmn).sqrt();
        tnn - sq_tmn / (d + d_sign * sqrt_term)
    } else {
        tnn
    }
}

/// Find the active subproblem `[start, end]` of the tridiagonal matrix, i.e. the
/// largest contiguous block whose off-diagonal entries are all above the
/// convergence threshold. Returns `(start, end)` (with `(0, 0)` meaning "done").
fn delimit_subproblem(
    diag: &[f64],
    mut off_diag: Vec<f64>,
    end: usize,
    eps: f64,
) -> (usize, usize, Vec<f64>) {
    let mut n = end;
    let mut searching = true;
    while n > 0 && searching {
        let m = n - 1;
        let od = abs_val(off_diag[m]);
        let dn = abs_val(diag[n]);
        let dm = abs_val(diag[m]);
        let is_large = od > eps * (dn + dm);
        if is_large {
            searching = false;
        } else {
            n = n - 1;
        }
    }

    if n == 0 {
        return (0, 0, off_diag);
    }

    let mut new_start = n - 1;
    let mut searching = true;
    while new_start > 0 && searching {
        let m = new_start - 1;
        let od = abs_val(off_diag[m]);
        let dn2 = abs_val(diag[new_start]);
        let dm2 = abs_val(diag[m]);
        let threshold = eps * (dn2 + dm2);
        let is_large = od > threshold;
        if is_large {
            new_start = new_start - 1;
        } else {
            off_diag[m] = 0.0;
            searching = false;
        }
    }

    (new_start, n, off_diag)
}

/// Compute the eigendecomposition of a real symmetric n×n matrix (column-major).
///
/// The algorithm is nalgebra's: Householder reduction to tridiagonal form,
/// followed by implicit QL iterations with Wilkinson shifts and Givens rotations
/// until all off-diagonal entries are within `eps` (or `max_niter` sweeps are
/// exhausted).
///
/// * `matrix` — flattened n×n symmetric matrix in column-major order
/// * `n` — dimension
/// * `eps` — convergence tolerance on the off-diagonal entries
/// * `max_niter` — maximum number of QL iterations
///
/// # Precondition
///
/// `matrix` must be symmetric (`matrix[i][j] == matrix[j][i]`). The eigenvalues
/// of a symmetric matrix are all real, which is what makes this decomposition
/// possible.  This precondition is stated in `Spec.lean` as an `IsSymmetric`
/// hypothesis; it is **not** checked at run time.
pub fn symmetric_eigen(
    matrix: &[f64],
    n: usize,
    eps: f64,
    max_niter: usize,
) -> SymmetricEigen {
    if n == 0 {
        return SymmetricEigen {
            eigenvalues: Vec::new(),
            eigenvectors: Vec::new(),
        };
    }

    // Householder reduction to tridiagonal form.
    let (tri, off_diagonal) = tridiagonalize(matrix.to_vec(), n);

    // Reconstruct Q and extract the (real) diagonal and off-diagonal of T.
    let mut q = assemble_q(&tri, &off_diagonal, n);

    let mut diag = vec![0.0; n];
    let mut i: usize = 0;
    while i < n {
        diag[i] = tri[i + i * n];
        i += 1;
    }

    let mut off_diag = vec![0.0; n - 1];
    let mut i: usize = 0;
    while i < n - 1 {
        off_diag[i] = abs_val(off_diagonal[i]);
        i += 1;
    }

    // Implicit QL iteration.
    let sub0 = delimit_subproblem(&diag, off_diag, n - 1, eps);
    let mut start = sub0.0;
    let mut end = sub0.1;
    let mut off_diag = sub0.2;

    let mut niter: usize = 0;
    while end != start && niter < max_niter {
        let subdim = end - start + 1;

        if subdim > 2 {
            // -- Implicit QL step on the subproblem [start, end] ------------
            let m = end - 1;
            let nn = end;
            let shift = wilkinson_shift(diag[m], diag[nn], off_diag[m]);

            let mut vx = diag[start] - shift;
            let mut vy = off_diag[start];

            let mut i = start;
            let mut chasing = true;
            while i < nn && chasing {
                let j = i + 1;

                let vy_zero = vy == 0.0;
                if vy_zero {
                    chasing = false;
                } else {
                    // Givens rotation that cancels the y-component of (vx, vy).
                    let mod0 = abs_val(vx);
                    let sign0 = if vx < 0.0 { -1.0 } else { 1.0 };
                    let denom = (mod0 * mod0 + vy * vy).sqrt();
                    let c = mod0 / denom;
                    let s = -vy / (sign0 * denom);
                    let r = sign0 * denom;

                    if i > start {
                        off_diag[i - 1] = r;
                    }

                    let mii = diag[i];
                    let mjj = diag[j];
                    let mij = off_diag[i];
                    let cc = c * c;
                    let ss = s * s;
                    let cs = c * s;
                    let b = cs * 2.0 * mij;
                    let new_di = cc * mii + ss * mjj - b;
                    let new_dj = ss * mii + cc * mjj + b;
                    let new_od = cs * (mii - mjj) + mij * (cc - ss);
                    diag[i] = new_di;
                    diag[j] = new_dj;
                    off_diag[i] = new_od;

                    if i != nn - 1 {
                        vx = off_diag[i];
                        vy = -s * off_diag[i + 1];
                        let new_od_next = off_diag[i + 1] * c;
                        off_diag[i + 1] = new_od_next;
                    }

                    // Accumulate the (inverse) Givens rotation into Q.
                    let mut k: usize = 0;
                    while k < n {
                        let a = q[k + i * n];
                        let bb = q[k + j * n];
                        let new_a = a * c - s * bb;
                        let new_b = s * a + c * bb;
                        q[k + i * n] = new_a;
                        q[k + j * n] = new_b;
                        k += 1;
                    }

                    i += 1;
                }
            }

            // Deflate if the trailing off-diagonal has converged.
            let dm = abs_val(diag[m]);
            let dnn = abs_val(diag[nn]);
            let odm = abs_val(off_diag[m]);
            let converged = odm <= eps * (dm + dnn);
            if converged {
                end = end - 1;
            }
        } else {
            // -- 2×2 subproblem: solve it directly --------------------------
            let a = diag[start];
            let b_od = off_diag[start];
            let c2 = diag[start + 1];
            let trace = a + c2;
            let diff = a - c2;
            let disc = (diff * diff + 4.0 * b_od * b_od).sqrt();
            let eig1 = (trace - disc) * 0.5;
            let eig2 = (trace + disc) * 0.5;

            // Choose the eigenvector basis least prone to cancellation.
            let d1 = eig1 - c2;
            let d2 = eig1 - a;
            let use_first = abs_val(d1) > abs_val(d2);
            let bx = if use_first { d1 } else { b_od };
            let by = if use_first { b_od } else { d2 };

            diag[start] = eig1;
            diag[start + 1] = eig2;

            // Rotate Q by the Givens rotation built from the eigenvector basis.
            let mod0 = abs_val(bx);
            let sign0 = if bx < 0.0 { -1.0 } else { 1.0 };
            let denom = (mod0 * mod0 + by * by).sqrt();
            if denom > eps {
                let c = mod0 / denom;
                let s = by / (sign0 * denom);
                let mut k: usize = 0;
                while k < n {
                    let a = q[k + start * n];
                    let bb = q[k + (start + 1) * n];
                    let new_a = a * c + s * bb;
                    let new_b = -s * a + c * bb;
                    q[k + start * n] = new_a;
                    q[k + (start + 1) * n] = new_b;
                    k += 1;
                }
            }

            end = end - 1;
        }

        // Re-delimit the subproblem (some decoupling may have occurred).
        let sub = delimit_subproblem(&diag, off_diag, end, eps);
        start = sub.0;
        end = sub.1;
        off_diag = sub.2;

        niter += 1;
    }

    SymmetricEigen {
        eigenvalues: diag,
        eigenvectors: q,
    }
}
