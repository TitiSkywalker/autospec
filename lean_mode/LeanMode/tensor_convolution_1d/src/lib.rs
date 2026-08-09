//! 1D convolution of f64 slices.

/// Compute the full 1D convolution of `input` with `kernel`.
///
/// Input length N, kernel length K, output length N + K - 1.
/// Returns an empty vector if either argument is empty.
pub fn conv1d(input: &[f64], kernel: &[f64]) -> Vec<f64> {
    let n = input.len();
    let k = kernel.len();
    if n == 0 || k == 0 {
        return vec![];
    }
    let out_len = n + k - 1;
    let mut out = vec![0.0; out_len];
    for i in 0..out_len {
        for j in 0..k {
            if i >= j && i - j < n {
                out[i] += input[i - j] * kernel[j];
            }
        }
    }
    out
}
