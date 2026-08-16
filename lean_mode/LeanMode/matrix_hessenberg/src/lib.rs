//! Hessenberg decomposition using Householder reflections.
//!
//! Extracted from nalgebra's `Hessenberg::new_with_workspace` + `unpack_h` + `q`
//! (via `householder::clear_column_unchecked` / `assemble_q`) and refined for
//! Aeneas compatibility (while loops, index-based access, no iterators/closures).
//!
//! Decomposes an n×n matrix A into A = Q · H · Qᵀ, where Q is orthogonal and H
//! is upper Hessenberg (every entry below the first subdiagonal is zero).
//!
//! All matrices are stored in **column-major** order: element (row, col) is
//! at index `row + col * n`.

/// Result of the Hessenberg decomposition.
pub struct Hessenberg {
    /// Orthogonal similarity transformation matrix Q (n×n, column-major).
    pub q: Vec<f64>,
    /// Upper Hessenberg matrix H (n×n, column-major).
    pub h: Vec<f64>,
}

/// Compute the Householder reflection axis for a vector, in place.
///
/// Given `axis[0..len]`, computes the reflection that zeros out all entries
/// except the first.  The vector is normalized to unit length.
///
/// Returns `(signed_norm, not_zero)` where `signed_norm` is the signed
/// Euclidean norm of the original vector.
fn reflection_axis(axis: &mut Vec<f64>, len: usize) -> (f64, bool) {
    if len == 0 {
        return (0.0, false);
    }

    // Squared Euclidean norm of the vector.
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

    // ‖v‖² = 2·(‖x‖² + |x₀|·‖x‖) for v = x + sign(x₀)·‖x‖·e₁.
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

/// Apply the two-sided Householder similarity transformation at step `k`.
///
/// For the unit reflector axis `v` (length m = n - k - 1) and sign
/// `s = signum(reflection_norm)`, applies
///   A[k+1..n, k+1..n] ← s² · H · A[k+1..n, k+1..n] · H = H · A · H
/// (with H = I - 2·v·vᵀ) plus the corresponding one-sided updates to the
/// border blocks.  This is the standard Hessenberg reduction step.
fn apply_reflection(
    mut hess: Vec<f64>,
    n: usize,
    k: usize,
    axis: &[f64],
    sign: f64,
) -> Vec<f64> {
    let m = n - k - 1;
    let two = -2.0 * sign;

    // Right multiply: hess[*, k+1..n] ← sign · hess[*, k+1..n] - 2·sign · (hess·v)·vᵀ
    let mut row: usize = 0;
    while row < n {
        let mut w: f64 = 0.0;
        let mut t: usize = 0;
        while t < m {
            w = w + hess[row + (k + 1 + t) * n] * axis[t];
            t += 1;
        }
        let mut col: usize = k + 1;
        while col < n {
            let idx = row + col * n;
            let new_val = sign * hess[idx] + two * w * axis[col - (k + 1)];
            hess[idx] = new_val;
            col += 1;
        }
        row += 1;
    }

    // Left multiply: hess[k+1..n, k+1..n] ← sign · hess - 2·sign · v·(vᵀ·hess)
    let mut col: usize = k + 1;
    while col < n {
        let mut d: f64 = 0.0;
        let mut t: usize = 0;
        while t < m {
            d = d + axis[t] * hess[(k + 1 + t) + col * n];
            t += 1;
        }
        let mut u: usize = 0;
        while u < m {
            let r = k + 1 + u;
            let idx = r + col * n;
            let new_val = sign * hess[idx] + two * d * axis[u];
            hess[idx] = new_val;
            u += 1;
        }
        col += 1;
    }

    hess
}

/// Perform one reduction step: zero out the subdiagonal of column `k` with a
/// Householder reflection and apply the two-sided transformation.
///
/// Returns the signed norm (the value stored in `subdiag[k]`) and the updated
/// compact matrix.
fn reduce_step(mut hess: Vec<f64>, n: usize, k: usize) -> (f64, Vec<f64>) {
    let m = n - k - 1;

    // Extract the sub-column of column `k` below the diagonal (rows k+1..n).
    let mut axis = vec![0.0; m];
    let mut t: usize = 0;
    while t < m {
        axis[t] = hess[(k + 1 + t) + k * n];
        t += 1;
    }

    let (reflection_norm, not_zero) = reflection_axis(&mut axis, m);

    // Store the (unit) Householder vector back into column `k`.
    let mut t: usize = 0;
    while t < m {
        hess[(k + 1 + t) + k * n] = axis[t];
        t += 1;
    }

    if not_zero {
        let sign = if reflection_norm < 0.0 { -1.0 } else { 1.0 };
        hess = apply_reflection(hess, n, k, &axis, sign);
    }

    (reflection_norm, hess)
}

/// Reconstruct the orthogonal factor Q (n×n, column-major) by applying the
/// Householder reflections to the identity in reverse order.
fn assemble_q(hess: &[f64], subdiag: &[f64], n: usize) -> Vec<f64> {
    let mut q = vec![0.0; n * n];
    let mut i: usize = 0;
    while i < n {
        q[i + i * n] = 1.0;
        i += 1;
    }

    // Apply reflections in reverse order: i = n-2 down to 0.
    let mut ii: usize = n - 1;
    while ii > 0 {
        ii -= 1;
        let i = ii;
        let m = n - i - 1;

        let sign = if subdiag[i] < 0.0 { -1.0 } else { 1.0 };
        let two = -2.0 * sign;

        // Reflect rows i+1..n of columns i..n by H = I - 2·v·vᵀ.
        let mut col: usize = i;
        while col < n {
            let mut dot: f64 = 0.0;
            let mut t: usize = 0;
            while t < m {
                dot = dot + hess[(i + 1 + t) + i * n] * q[(i + 1 + t) + col * n];
                t += 1;
            }
            let mut u: usize = 0;
            while u < m {
                let r = i + 1 + u;
                let idx = r + col * n;
                let new_val = sign * q[idx] + two * dot * hess[(i + 1 + u) + i * n];
                q[idx] = new_val;
                u += 1;
            }
            col += 1;
        }
    }

    q
}

/// Reconstruct the upper Hessenberg matrix H (n×n, column-major) from the
/// compact form: upper part (including diagonal) taken from `hess`, first
/// subdiagonal set to |subdiag[k]|, everything below that zeroed.
fn unpack_h(hess: &[f64], subdiag: &[f64], n: usize) -> Vec<f64> {
    let mut h = vec![0.0; n * n];

    // Copy the upper triangle (i <= j).
    let mut col: usize = 0;
    while col < n {
        let mut row: usize = 0;
        while row <= col {
            h[row + col * n] = hess[row + col * n];
            row += 1;
        }
        col += 1;
    }

    // Set the subdiagonal entries to |subdiag[k]|.
    let mut k: usize = 0;
    while k < n - 1 {
        let sd = subdiag[k];
        let abs_sd = if sd < 0.0 { -sd } else { sd };
        h[(k + 1) + k * n] = abs_sd;
        k += 1;
    }

    h
}

/// Compute the Hessenberg decomposition of an n×n matrix (column-major).
///
/// Returns Q (n×n, orthogonal) and H (n×n, upper Hessenberg) such that
/// A = Q · H · Qᵀ.
///
/// * `matrix` — flattened n×n matrix in column-major order
/// * `n` — the dimension of the square matrix
pub fn hessenberg_decomposition(matrix: &[f64], n: usize) -> Hessenberg {
    if n == 0 {
        return Hessenberg {
            q: Vec::new(),
            h: Vec::new(),
        };
    }

    // Compute the compact form (Householder vectors + signed norms).
    let mut hess = matrix.to_vec();
    let mut subdiag: Vec<f64> = vec![0.0; n - 1];

    let mut k: usize = 0;
    while k < n - 1 {
        let (norm, hess2) = reduce_step(hess, n, k);
        hess = hess2;
        subdiag[k] = norm;
        k += 1;
    }

    // Reconstruct the explicit Q and H factors.
    let q = assemble_q(&hess, &subdiag, n);
    let h = unpack_h(&hess, &subdiag, n);

    Hessenberg { q, h }
}
