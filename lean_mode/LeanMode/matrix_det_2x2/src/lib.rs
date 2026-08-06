//! 2×2 matrix determinant.

/// Compute the determinant of a 2×2 matrix stored in row-major order.
///
/// `matrix` is `[a, b, c, d]` representing `[[a, b], [c, d]]`.
/// Returns `a * d - b * c`.
pub fn det_2x2(matrix: &[f64]) -> f64 {
    let a = matrix[0];
    let b = matrix[1];
    let c = matrix[2];
    let d = matrix[3];
    let det = a * d - b * c;
    det
}
