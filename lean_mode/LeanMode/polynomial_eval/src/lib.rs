//! Evaluate a polynomial at a given point `x`.
//!
//! The polynomial is represented as a slice of coefficients in ascending order:
//! `[a₀, a₁, a₂, ...]` represents `a₀ + a₁·x + a₂·x² + ...`.
//!
//! The coefficient slice must be non-empty.

/// Evaluate a polynomial for some value `x`.
///
/// Coefficients are in ascending order (constant term first).
/// The slice must have at least one element.
pub fn polynomial_eval(coeffs: &[f64], x: f64) -> f64 {
    let len = coeffs.len();
    assert!(len > 0);
    let mut p = x;
    let mut res = coeffs[0];
    let mut i: usize = 1;
    while i < len {
        res = res + p * coeffs[i];
        p = p * x;
        i = i + 1;
    }
    res
}
