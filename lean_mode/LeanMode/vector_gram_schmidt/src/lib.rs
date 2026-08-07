//! Single step of Gram-Schmidt orthogonalization.
//!
//! Given two vectors u and v, compute:
//!
//! ```text
//! v' = v - (⟨v, u⟩ / ⟨u, u⟩) · u
//! ```
//!
//! The resulting vector v' is orthogonal to u.
//!
//! # Panics
//!
//! Panics if u is the zero vector (‖u‖ = 0).

/// Compute one Gram-Schmidt step: orthogonalize v against u.
///
/// * `u` — reference vector (must be non-zero)
/// * `v` — vector to orthogonalize
/// * `n` — length of both vectors
///
/// Returns `v' = v - proj_u(v)` such that ⟨v', u⟩ = 0.
pub fn gram_schmidt_step(u: &[f64], v: &[f64], n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }

    // Compute ⟨u, u⟩ and ⟨v, u⟩.
    let mut dot_uu: f64 = 0.0;
    let mut dot_vu: f64 = 0.0;
    let mut i: usize = 0;
    while i < n {
        dot_uu = dot_uu + u[i] * u[i];
        dot_vu = dot_vu + v[i] * u[i];
        i += 1;
    }

    assert!(dot_uu != 0.0, "zero vector");

    let factor = dot_vu / dot_uu;

    let mut result: Vec<f64> = Vec::new();
    let mut i: usize = 0;
    while i < n {
        result.push(v[i] - factor * u[i]);
        i += 1;
    }

    result
}
