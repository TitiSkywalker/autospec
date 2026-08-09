//! Population variance of f64 slice.

/// Return the population variance of all elements in `input`.
///
/// Returns `0.0` for an empty slice.
pub fn var(input: &[f64]) -> f64 {
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
    let mean = total / count;
    let mut sum_sq = 0.0;
    for i in 0..n {
        let diff = input[i] - mean;
        sum_sq += diff * diff;
    }
    sum_sq / count
}
