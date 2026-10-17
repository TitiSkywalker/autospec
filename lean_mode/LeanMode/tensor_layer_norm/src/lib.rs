//! 2D layer normalization (row-wise, row-major storage).

/// Layer-normalize each row of an H×W matrix stored in row-major order.
///
/// For each row r:
///   μ_r  = (1/W) · Σ_j x[r][j]
///   σ²_r = (1/W) · Σ_j (x[r][j] - μ_r)²
///   y[r][j] = γ[j] · (x[r][j] - μ_r) / √(σ²_r + ε) + β[j]
///
/// γ and β are per-feature affine parameters of length W.
///
/// `eps` must be positive: it stabilizes `√(σ² + ε)` against zero variance.
///
/// Returns an empty vector if either dimension is zero.
pub fn layer_norm(
    input: &[f64],
    rows: usize,
    cols: usize,
    gamma: &[f64],
    beta: &[f64],
    eps: f64,
) -> Vec<f64> {
    if rows == 0 || cols == 0 {
        return vec![];
    }
    let len = rows * cols;
    let mut out = vec![0.0; len];

    // W as an f64 (usize → f64 casts are not translatable, so count 1.0s).
    let mut wf = 0.0;
    let mut c: usize = 0;
    while c < cols {
        wf = wf + 1.0;
        c = c + 1;
    }

    let mut r: usize = 0;
    while r < rows {
        let base = r * cols;

        // Per-row mean.
        let mut sum = 0.0;
        let mut j: usize = 0;
        while j < cols {
            sum = sum + input[base + j];
            j = j + 1;
        }
        let mean = sum / wf;

        // Per-row variance.
        let mut sumsq = 0.0;
        j = 0;
        while j < cols {
            let d = input[base + j] - mean;
            sumsq = sumsq + d * d;
            j = j + 1;
        }
        let var = sumsq / wf;

        // Normalize and apply the affine transform.
        let stddev = (var + eps).sqrt();
        j = 0;
        while j < cols {
            let xhat = (input[base + j] - mean) / stddev;
            out[base + j] = gamma[j] * xhat + beta[j];
            j = j + 1;
        }

        r = r + 1;
    }
    out
}
