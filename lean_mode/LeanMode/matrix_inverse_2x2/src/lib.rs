//! 2x2 matrix inverse — computes the inverse using the determinant formula.

/// Invert a 2x2 matrix (row-major: `[a, b, c, d]` for `[[a,b],[c,d]]`).
///
/// Panics if the determinant is zero (singular matrix).
/// Otherwise returns the inverse in row-major order.
pub fn invert_2x2(matrix: &[f64]) -> Vec<f64> {
    let a = matrix[0];
    let b = matrix[1];
    let c = matrix[2];
    let d = matrix[3];
    let det = a * d - b * c;
    assert!(det != 0.0, "singular matrix");
    let inv_det = 1.0 / det;
    vec![d * inv_det, -b * inv_det, -c * inv_det, a * inv_det]
}
