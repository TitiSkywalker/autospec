//! 1D softmax of f64 slices.

/// Compute the softmax of `input`.
///
/// softmax(x)[i] = exp(x[i]) / Σⱼ exp(x[j])
///
/// Returns a vector of zeros if the input is empty.
pub fn softmax(input: &[f64]) -> Vec<f64> {
    let n = input.len();
    if n == 0 {
        return vec![];
    }

    // Compute exp(x[i]) and their sum.
    let mut sum = 0.0;
    let mut out = vec![0.0; n];
    for i in 0..n {
        let e = input[i].exp();
        sum += e;
        out[i] = e;
    }

    // Normalize.
    for i in 0..n {
        out[i] = out[i] / sum;
    }
    out
}
