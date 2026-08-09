//! Mean of f64 slice.

/// Return the arithmetic mean of all elements in `input`.
///
/// Returns `0.0` for an empty slice.
pub fn mean(input: &[f64]) -> f64 {
    let n = input.len();
    if n == 0 {
        return 0.0;
    }
    let mut total = 0.0;
    let mut count = 0.0;
    for i in 0..n {
        total += input[i];
        count += 1.0;
    }
    total / count
}
