//! Integer-exponent matrix power (row-major storage).

/// Multiply two n×n row-major matrices, returning a new n×n row-major matrix.
fn mat_mul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut c = vec![0.0; n * n];
    let mut i: usize = 0;
    while i < n {
        let mut j: usize = 0;
        while j < n {
            let mut sum: f64 = 0.0;
            let mut k: usize = 0;
            while k < n {
                sum = sum + a[i * n + k] * b[k * n + j];
                k += 1;
            }
            c[i * n + j] = sum;
            j += 1;
        }
        i += 1;
    }
    c
}

/// n×n identity matrix (row-major).
fn identity(n: usize) -> Vec<f64> {
    let mut id = vec![0.0; n * n];
    let mut i: usize = 0;
    while i < n {
        id[i * n + i] = 1.0;
        i += 1;
    }
    id
}

/// Raise a row-major n×n matrix to an integer power via exponentiation by squaring.
///
/// * `matrix` — flattened n×n matrix in row-major order
/// * `n` — dimension
/// * `exp` — non-negative integer exponent (`matrix^0` is the identity)
///
/// Returns `matrix^exp` as an n×n row-major `Vec`.
pub fn matrix_pow(matrix: &[f64], n: usize, exp: usize) -> Vec<f64> {
    if exp == 0 {
        return identity(n);
    }

    let mut result = identity(n);
    let mut base = matrix.to_vec();
    let mut e = exp;

    while e > 0 {
        if e % 2 == 1 {
            result = mat_mul(&result, &base, n);
        }
        e = e / 2;
        if e > 0 {
            base = mat_mul(&base, &base, n);
        }
    }

    result
}
