//! n×n matrix Jordan normal form, computed exactly over the rationals.
//!
//! Given an n×n matrix A with rational (in practice integer) entries, computes
//! the Jordan normal form J of A: a block-diagonal matrix whose blocks are
//! Jordan blocks J_m(λ) (λ on the diagonal, 1 on the superdiagonal).
//!
//! Algorithm:
//!   1. characteristic polynomial det(x·I − A) via Faddeev–LeVerrier;
//!   2. find its integer roots (bounded by the row-sum Gershgorin bound);
//!   3. for each eigenvalue λ, rank of (A − λ·I)^j for j = 0..n determines the
//!      Jordan block sizes (dimension of the generalized eigenspaces);
//!   4. assemble the block-diagonal Jordan matrix.
//!
//! `success` is `true` exactly when every eigenvalue is an integer.  For an
//! integer-entry matrix the characteristic polynomial is a monic integer
//! polynomial, whose rational roots are automatically integers, so this is
//! equivalent to the characteristic polynomial splitting over ℚ; but a
//! non-integer rational eigenvalue (only possible for fractional matrix
//! entries) is also rejected.  All arithmetic is exact rational.
//!
//! All matrices are stored in row-major order.

/// Result of the Jordan normal form computation.
pub struct Jordan {
    /// The n×n Jordan matrix (row-major); all zeros when `success` is `false`.
    pub j: Vec<f64>,
    /// Whether every eigenvalue is an integer (char poly splits over ℤ).
    pub success: bool,
}

/// Multiply two n×n matrices (row-major).
fn mat_mul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n * n];
    let mut row = 0;
    while row < n {
        let mut col = 0;
        while col < n {
            let mut sum = 0.0;
            let mut t = 0;
            while t < n {
                sum = sum + a[row * n + t] * b[t * n + col];
                t += 1;
            }
            out[row * n + col] = sum;
            col += 1;
        }
        row += 1;
    }
    out
}

/// Trace of an n×n matrix (row-major).
fn trace(a: &[f64], n: usize) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;
    while i < n {
        sum = sum + a[i * n + i];
        i += 1;
    }
    sum
}

/// Characteristic polynomial det(x·I − A) via Faddeev–LeVerrier.
///
/// Returns `[c₀, c₁, …, cₙ]` in ascending degree order (cₙ = 1).
fn charpoly(matrix: &[f64], n: usize) -> Vec<f64> {
    let mut b = vec![0.0; n * n];
    let mut i = 0;
    while i < n {
        b[i * n + i] = 1.0;
        i += 1;
    }

    let mut coeffs = vec![0.0; n + 1];
    coeffs[n] = 1.0;

    let mut k = 1;
    let mut kf = 1.0;
    while k <= n {
        let c = mat_mul(matrix, &b, n);
        let t = trace(&c, n);
        let pk = (0.0 - t) / kf; // cₙ₋ₖ = −tr(A·Bₖ)/k
        coeffs[n - k] = pk;

        if k < n {
            let mut idx = 0;
            while idx < n * n {
                b[idx] = c[idx];
                idx += 1;
            }
            let mut d = 0;
            while d < n {
                let diag = d * n + d;
                let sum = c[diag] + pk;
                b[diag] = sum;
                d += 1;
            }
        }

        k += 1;
        kf = kf + 1.0;
    }

    coeffs
}

/// Evaluate `c₀ + c₁·x + ⋯ + c_deg·x^deg` (ascending coefficients) by Horner.
fn eval_poly(coeffs: &[f64], deg: usize, x: f64) -> f64 {
    let mut result = coeffs[deg];
    let mut k = deg;
    while k > 0 {
        k -= 1;
        result = result * x + coeffs[k];
    }
    result
}

/// Divide a degree-`deg` polynomial (ascending coefficients) by `(x − d)`,
/// returning the ascending coefficients of the quotient of degree `deg − 1`.
/// The caller guarantees that `(x − d)` divides the input.
fn divide_linear(coeffs: &[f64], deg: usize, d: f64) -> Vec<f64> {
    let mut q = vec![0.0; deg];
    q[deg - 1] = coeffs[deg];
    let mut k = deg - 1;
    while k > 0 {
        let term = d * q[k];
        let sum = coeffs[k] + term;
        q[k - 1] = sum;
        k -= 1;
    }
    q
}

/// Gershgorin row-sum bound: every eigenvalue λ satisfies |λ| ≤ maxᵢ Σⱼ |aᵢⱼ|.
fn row_sum_bound(matrix: &[f64], n: usize) -> f64 {
    let mut max_sum = 0.0;
    let mut i = 0;
    while i < n {
        let mut sum = 0.0;
        let mut j = 0;
        while j < n {
            let v = matrix[i * n + j];
            let a = if v < 0.0 { -v } else { v };
            sum = sum + a;
            j += 1;
        }
        let cond = sum > max_sum;
        if cond {
            max_sum = sum;
        }
        i += 1;
    }
    max_sum
}

/// Rank of an nrows×ncols matrix (row-major) via Gaussian elimination with
/// partial pivoting.
fn rank(matrix: &[f64], nrows: usize, ncols: usize) -> usize {
    let mut a = matrix.to_vec();
    let mut rk: usize = 0;
    let mut col: usize = 0;

    while col < ncols && rk < nrows {
        let mut pivot_row = rk;
        let mut max_val = a[rk * ncols + col];
        if max_val < 0.0 {
            max_val = -max_val;
        }
        let mut r = rk + 1;
        while r < nrows {
            let mut v = a[r * ncols + col];
            if v < 0.0 {
                v = -v;
            }
            if v > max_val {
                max_val = v;
                pivot_row = r;
            }
            r += 1;
        }

        if max_val == 0.0 {
            col += 1;
        } else {
            if pivot_row != rk {
                let mut j = col;
                while j < ncols {
                    let tmp = a[rk * ncols + j];
                    a[rk * ncols + j] = a[pivot_row * ncols + j];
                    a[pivot_row * ncols + j] = tmp;
                    j += 1;
                }
            }

            let pivot = a[rk * ncols + col];
            let mut r = rk + 1;
            while r < nrows {
                let factor = a[r * ncols + col] / pivot;
                let mut j = col + 1;
                while j < ncols {
                    a[r * ncols + j] = a[r * ncols + j] - factor * a[rk * ncols + j];
                    j += 1;
                }
                r += 1;
            }

            rk += 1;
            col += 1;
        }
    }

    rk
}

/// n×n identity matrix (row-major).
fn identity(n: usize) -> Vec<f64> {
    let mut m = vec![0.0; n * n];
    let mut i = 0;
    while i < n {
        m[i * n + i] = 1.0;
        i += 1;
    }
    m
}

/// A − λ·I (row-major).
fn mat_sub_identity_times(matrix: &[f64], n: usize, lambda: f64) -> Vec<f64> {
    let mut m = matrix.to_vec();
    let mut i = 0;
    while i < n {
        let diag = i * n + i;
        let sub = m[diag] - lambda;
        m[diag] = sub;
        i += 1;
    }
    m
}

/// Jordan normal form of an n×n matrix (row-major).
pub fn jordan_form(matrix: &[f64], n: usize) -> Jordan {
    if n == 0 {
        return Jordan {
            j: Vec::new(),
            success: true,
        };
    }

    // 1–2. Characteristic polynomial and its integer-root bound.
    let mut coeffs = charpoly(matrix, n); // ascending, length n+1
    let b = row_sum_bound(matrix, n);

    // Factor det(x·I − A) into linear factors (x − λ) with integer λ.
    let mut deg = n;
    let mut roots = vec![0.0; n];
    let mut nroots = 0;

    let mut d = 0.0;
    while d <= b {
        // Factor out (x − d) with multiplicity.
        let mut stop = false;
        while !stop && deg > 0 {
            let val = eval_poly(&coeffs, deg, d);
            if val == 0.0 {
                coeffs = divide_linear(&coeffs, deg, d);
                roots[nroots] = d;
                nroots += 1;
                deg -= 1;
            } else {
                stop = true;
            }
        }
        // Factor out (x + d) with multiplicity.
        if d != 0.0 {
            let nd = 0.0 - d;
            let mut stop2 = false;
            while !stop2 && deg > 0 {
                let val = eval_poly(&coeffs, deg, nd);
                if val == 0.0 {
                    coeffs = divide_linear(&coeffs, deg, nd);
                    roots[nroots] = nd;
                    nroots += 1;
                    deg -= 1;
                } else {
                    stop2 = true;
                }
            }
        }
        d = d + 1.0;
    }

    // Some eigenvalue is not an integer — no integer Jordan normal form.
    if deg != 0 {
        return Jordan {
            j: vec![0.0; n * n],
            success: false,
        };
    }

    // Assemble the Jordan matrix from the (grouped) roots.
    let mut j = vec![0.0; n * n];
    let mut offset = 0;
    let mut i = 0;
    while i < nroots {
        let lambda = roots[i];

        // Find the end of this eigenvalue's (consecutive) run in `roots`.
        let mut jj = i + 1;
        let mut same = true;
        while same {
            if jj >= nroots {
                same = false;
            } else {
                if roots[jj] == lambda {
                    jj += 1;
                } else {
                    same = false;
                }
            }
        }

        // r[k] = rank((A − λ·I)^k) for k = 0..n (r[0] = n).
        let mut r = vec![0usize; n + 1];
        r[0] = n;
        let m1 = mat_sub_identity_times(matrix, n, lambda);
        let mut power = identity(n);
        let mut k = 1;
        while k <= n {
            power = mat_mul(&power, &m1, n);
            r[k] = rank(&power, n, n);
            k += 1;
        }

        // Number of Jordan blocks of size ≥ s is r[s−1] − r[s]; hence the
        // number of blocks of size exactly s is (r[s−1] − r[s]) − (r[s] − r[s+1]).
        let mut s = 1;
        while s <= n {
            let dj = r[s - 1] - r[s];
            let mut dnext = 0;
            if s < n {
                dnext = r[s] - r[s + 1];
            }
            let exact = dj - dnext;
            let mut t = 0;
            while t < exact {
                let mut a = 0;
                while a < s {
                    let diag_idx = (offset + a) * n + (offset + a);
                    j[diag_idx] = lambda;
                    if a + 1 < s {
                        let super_idx = (offset + a) * n + (offset + a + 1);
                        j[super_idx] = 1.0;
                    }
                    a += 1;
                }
                offset += s;
                t += 1;
            }
            s += 1;
        }

        i = jj;
    }

    Jordan { j, success: true }
}
