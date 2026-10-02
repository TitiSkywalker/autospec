//! Maximum of f64 slice.

/// Return the maximum element in `input`.
///
/// Returns `0.0` for an empty slice.
pub fn max(input: &[f64]) -> f64 {
    let n = input.len();
    if n == 0 {
        return 0.0;
    }
    let mut max_val = input[0];
    for i in 1..n {
        if input[i] > max_val {
            max_val = input[i];
        }
    }
    max_val
}
