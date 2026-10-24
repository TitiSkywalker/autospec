//! n×n matrix characteristic polynomial via the Faddeev–LeVerrier algorithm.
//!
//! Returns the n+1 coefficients of the characteristic polynomial
//! det(x·I − A) = c₀ + c₁·x + ⋯ + cₙ·xⁿ  in ascending degree order
//! (cₙ = 1).  Matrices are stored in row-major order.

/// Multiply two n×n matrices (row-major).
fn mat_mul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n * n];
    let mut row = 0;
    while row < n {
        let mut col = 0;
        while col < n {
            let mut sum = 0.0;
            let mut t = 0;
            while t < n {
                sum = sum + a[row * n + t] * b[t * n + col];
                t += 1;
            }
            out[row * n + col] = sum;
            col += 1;
        }
        row += 1;
    }
    out
}

/// Trace of an n×n matrix (row-major).
fn trace(a: &[f64], n: usize) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;
    while i < n {
        sum = sum + a[i * n + i];
        i += 1;
    }
    sum
}

/// Characteristic polynomial of an n×n matrix (row-major) via
/// Faddeev–LeVerrier.
///
/// Let p(x) = det(x·I − A) = c₀ + c₁·x + ⋯ + cₙ·xⁿ.  This returns
/// `[c₀, c₁, …, cₙ]` (so `cₙ = 1`).
pub fn charpoly(matrix: &[f64], n: usize) -> Vec<f64> {
    // Faddeev–LeVerrier fills the coefficients from the top down:
    //   B₁ = I
    //   cₙ₋₁ = −tr(A·B₁) / 1
    //   B₂ = A·B₁ + cₙ₋₁·I
    //   cₙ₋₂ = −tr(A·B₂) / 2
    //   ⋯
    // with Bₖ₊₁ = A·Bₖ + cₙ₋ₖ·I and cₙ₋ₖ = −tr(A·Bₖ)/k.
    // Since cₙ₋ₖ lands at index n−k, `coeffs` ends up in ascending order.

    let mut b = vec![0.0; n * n];
    let mut i = 0;
    while i < n {
        b[i * n + i] = 1.0;
        i += 1;
    }

    let mut coeffs = vec![0.0; n + 1];
    coeffs[n] = 1.0;

    let mut k = 1;
    let mut kf = 1.0;
    while k <= n {
        let c = mat_mul(matrix, &b, n);
        let t = trace(&c, n);
        let pk = (0.0 - t) / kf; // cₙ₋ₖ = −tr(A·Bₖ)/k
        coeffs[n - k] = pk;

        if k < n {
            // Bₖ₊₁ = A·Bₖ + cₙ₋ₖ·I = c + pk·I
            let mut idx = 0;
            while idx < n * n {
                b[idx] = c[idx];
                idx += 1;
            }
            let mut d = 0;
            while d < n {
                let diag = d * n + d;
                let sum = c[diag] + pk;
                b[diag] = sum;
                d += 1;
            }
        }

        k += 1;
        kf = kf + 1.0;
    }

    coeffs
}
