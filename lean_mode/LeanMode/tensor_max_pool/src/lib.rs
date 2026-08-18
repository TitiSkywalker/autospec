//! 2D max pooling of f64 slices (row-major storage).

/// Compute the 2D max pooling of `input` with a `kh`×`kw` window (stride 1, no padding).
///
/// Input is H×W, output is (H-kh+1)×(W-kw+1), both row-major.
///
/// `out[oh,ow] = max over i in [0,kh), j in [0,kw) of input[oh+i][ow+j]`.
///
/// Returns an empty vector if any dimension is zero or the window is larger
/// than the input (`kh > H` or `kw > W`).
pub fn max_pool2d(
    input: &[f64],
    h: usize,
    w: usize,
    kh: usize,
    kw: usize,
) -> Vec<f64> {
    if h == 0 || w == 0 || kh == 0 || kw == 0 || kh > h || kw > w {
        return vec![];
    }
    let out_h = h - kh + 1;
    let out_w = w - kw + 1;
    let mut out = vec![0.0; out_h * out_w];
    for oh in 0..out_h {
        for ow in 0..out_w {
            // Initialize to the first window element (so all-negative windows work).
            let mut m = input[oh * w + ow];
            for i in 0..kh {
                for j in 0..kw {
                    let v = input[(oh + i) * w + (ow + j)];
                    if v > m {
                        m = v;
                    }
                }
            }
            out[oh * out_w + ow] = m;
        }
    }
    out
}