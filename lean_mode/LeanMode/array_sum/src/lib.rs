//! Sum of f64 slice.

/// Return the sum of all elements in `input`.
///
/// Returns `0.0` for an empty slice.
pub fn sum(input: &[f64]) -> f64 {
    let mut total = 0.0;
    for i in 0..input.len() {
        total += input[i];
    }
    total
}
