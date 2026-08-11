//! 2D full convolution of f64 slices (row-major storage).

/// Compute the full 2D convolution of `input` with `kernel`.
///
/// Input is H×W, kernel is Kh×Kw, both in row-major order.
/// Output is (H + Kh - 1) × (W + Kw - 1), row-major.
///
/// Returns an empty vector if either argument is empty or a dimension is zero.
pub fn conv2d(
    input: &[f64],
    h: usize,
    w: usize,
    kernel: &[f64],
    kh: usize,
    kw: usize,
) -> Vec<f64> {
    if h == 0 || w == 0 || kh == 0 || kw == 0 {
        return vec![];
    }
    let out_h = h + kh - 1;
    let out_w = w + kw - 1;
    let out_len = out_h * out_w;
    let mut out = vec![0.0; out_len];
    for i in 0..out_h {
        for j in 0..out_w {
            for m in 0..kh {
                for n in 0..kw {
                    if i >= m && i - m < h && j >= n && j - n < w {
                        out[i * out_w + j] +=
                            input[(i - m) * w + (j - n)] * kernel[m * kw + n];
                    }
                }
            }
        }
    }
    out
}
