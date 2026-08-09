//! Euclidean norm (L² norm) of a vector.

/// Compute `‖v‖ = sqrt(Σ v[i]²)`.
pub fn vector_norm(v: &[f64], n: usize) -> f64 {
    let mut sum_sq: f64 = 0.0;
    let mut i: usize = 0;
    while i < n {
        sum_sq = sum_sq + v[i] * v[i];
        i = i + 1;
    }
    sum_sq.sqrt()
}
