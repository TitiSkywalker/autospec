//! Smith normal form of an integer matrix.
//!
//! Every integer matrix `A` can be written as
//!
//! ```text
//!   D = P · A · Q
//! ```
//!
//! where `P` and `Q` are *unimodular* (integer, `det = ±1`) and `D` is
//! diagonal with non-negative entries satisfying the divisibility chain
//!
//! ```text
//!   d₁ | d₂ | … | d_r        (r = min(nrows, ncols))
//! ```
//!
//! The `dᵢ` are the *invariant factors* of `A` — the structure theorem for
//! finitely generated abelian groups is exactly this normal form.
//!
//! ## Algorithm
//!
//! For `t = 0, 1, …` the entry `(t, t)` is used as a pivot:
//!
//! 1. move some non-zero entry of the trailing sub-matrix to `(t, t)`
//!    (row swap + column swap),
//! 2. clear column `t` below the pivot: for `i > t`,
//!    `row_i -= (a[i][t] / a[t][t]) · row_t`.  `a[i][t]` may still be non-zero
//!    when the division left a remainder; then the remainder (strictly smaller
//!    in absolute value than the pivot) is swapped into `(t, t)` and the pass
//!    is repeated,
//! 3. clear row `t` to the right of the pivot the same way, with column
//!    operations,
//! 4. **divisibility**: if some `a[i][j]` (`i, j > t`) is not divisible by
//!    `a[t][t]`, add `row_i` to `row_t` and repeat — the offending entry is
//!    then either cleared or swapped into the pivot position, decreasing it.
//!
//! Every retry strictly decreases `|a[t][t]|`, so each diagonal position is
//! reached after finitely many passes.  All row operations are accumulated in
//! `P` (left multiplication) and all column operations in `Q` (right
//! multiplication), so that at every point `P · A₀ · Q` is the current matrix;
//! at the end the first `min(nrows, ncols)` diagonal entries are flipped to be
//! non-negative (a row operation, tracked in `P`).
//!
//! Matrices are flat and row-major.

/// Flat index of `(row, col)` in a row-major matrix with `ncols` columns.
fn idx(row: usize, ncols: usize, col: usize) -> usize {
    let t = row * ncols;
    let r = t + col;
    r
}

/// Result of the Smith normal form computation.
///
/// `d` is the `nrows × ncols` Smith normal form (diagonal, non-negative
/// diagonal, divisibility chain), and `p` (`nrows × nrows`) and `q`
/// (`ncols × ncols`) are the unimodular transforms with `d = p · a · q`.
pub struct SmithForm {
    pub p: Vec<i64>,
    pub d: Vec<i64>,
    pub q: Vec<i64>,
}

/// Smith normal form of the `nrows × ncols` matrix `matrix`, together with the
/// unimodular transforms: returns `SmithForm { p, d, q }` with
/// `d = p · matrix · q`, `d` diagonal, `d[i][i] ≥ 0` and
/// `d[i][i] | d[i+1][i+1]` for every `i`.
pub fn smith_normal_form(matrix: &[i64], nrows: usize, ncols: usize) -> SmithForm {
    // The working matrix `a` (starts as `matrix`), and the two accumulators.
    let mut a = matrix.to_vec();
    let mut p = vec![0i64; nrows * nrows];
    let mut q = vec![0i64; ncols * ncols];

    let mut z: usize = 0;
    while z < nrows {
        p[idx(z, nrows, z)] = 1;
        z += 1;
    }
    let mut z2: usize = 0;
    while z2 < ncols {
        q[idx(z2, ncols, z2)] = 1;
        z2 += 1;
    }

    let nm = if nrows < ncols { nrows } else { ncols };

    let mut t: usize = 0;
    while t < nm {
        let mut advanced = false;
        while !advanced {
            // 1. First non-zero entry of the trailing sub-matrix, kept in
            //    `piv = [linear position, row, column]` (a zero `nrows * ncols`
            //    linear position means "no candidate": the sub-matrix is zero).
            let mut piv: Vec<usize> = Vec::new();
            piv.push(nrows * ncols);
            piv.push(nrows);
            piv.push(ncols);
            let mut r: usize = t;
            while r < nrows {
                let mut c: usize = t;
                while c < ncols {
                    let v = a[idx(r, ncols, c)];
                    if v != 0 {
                        let lin = r * ncols + c;
                        let cur = piv[0];
                        if lin < cur {
                            piv[0] = lin;
                            piv[1] = r;
                            piv[2] = c;
                        }
                    }
                    c += 1;
                }
                r += 1;
            }
            let pr = piv[1];
            let pc = piv[2];

            if pr == nrows {
                // The trailing sub-matrix is identically zero: done.
                t = nm;
                advanced = true;
            } else {
                // Move the candidate to position (t, t).
                if pr != t {
                    let mut c: usize = 0;
                    while c < ncols {
                        let tmp = a[idx(t, ncols, c)];
                        a[idx(t, ncols, c)] = a[idx(pr, ncols, c)];
                        a[idx(pr, ncols, c)] = tmp;
                        c += 1;
                    }
                    let mut c2: usize = 0;
                    while c2 < nrows {
                        let tmp = p[idx(t, nrows, c2)];
                        p[idx(t, nrows, c2)] = p[idx(pr, nrows, c2)];
                        p[idx(pr, nrows, c2)] = tmp;
                        c2 += 1;
                    }
                }
                if pc != t {
                    let mut r2: usize = 0;
                    while r2 < nrows {
                        let tmp = a[idx(r2, ncols, t)];
                        a[idx(r2, ncols, t)] = a[idx(r2, ncols, pc)];
                        a[idx(r2, ncols, pc)] = tmp;
                        r2 += 1;
                    }
                    let mut c3: usize = 0;
                    while c3 < ncols {
                        let tmp = q[idx(c3, ncols, t)];
                        q[idx(c3, ncols, t)] = q[idx(c3, ncols, pc)];
                        q[idx(c3, ncols, pc)] = tmp;
                        c3 += 1;
                    }
                }

                // 2./3. Clear column `t` below the pivot and row `t` to its
                //      right; `clean` stays `true` only if both succeeded.
                let pivot = a[idx(t, ncols, t)];
                let mut clean = true;

                let mut i: usize = t + 1;
                while i < nrows {
                    if clean {
                        let v = a[idx(i, ncols, t)];
                        if v != 0 {
                            let f = v / pivot;
                            let mut c: usize = 0;
                            while c < ncols {
                                let sub = f * a[idx(t, ncols, c)];
                                let nv = a[idx(i, ncols, c)] - sub;
                                a[idx(i, ncols, c)] = nv;
                                c += 1;
                            }
                            let mut c2: usize = 0;
                            while c2 < nrows {
                                let sub = f * p[idx(t, nrows, c2)];
                                let nv = p[idx(i, nrows, c2)] - sub;
                                p[idx(i, nrows, c2)] = nv;
                                c2 += 1;
                            }
                            let left = a[idx(i, ncols, t)];
                            if left != 0 {
                                // Move the (smaller) remainder into (t, t).
                                let mut c3: usize = 0;
                                while c3 < ncols {
                                    let tmp = a[idx(t, ncols, c3)];
                                    a[idx(t, ncols, c3)] = a[idx(i, ncols, c3)];
                                    a[idx(i, ncols, c3)] = tmp;
                                    c3 += 1;
                                }
                                let mut c4: usize = 0;
                                while c4 < nrows {
                                    let tmp = p[idx(t, nrows, c4)];
                                    p[idx(t, nrows, c4)] = p[idx(i, nrows, c4)];
                                    p[idx(i, nrows, c4)] = tmp;
                                    c4 += 1;
                                }
                                clean = false;
                            }
                        }
                    }
                    i += 1;
                }

                let mut j: usize = t + 1;
                while j < ncols {
                    if clean {
                        let v = a[idx(t, ncols, j)];
                        if v != 0 {
                            let f = v / pivot;
                            let mut r3: usize = 0;
                            while r3 < nrows {
                                let sub = f * a[idx(r3, ncols, t)];
                                let nv = a[idx(r3, ncols, j)] - sub;
                                a[idx(r3, ncols, j)] = nv;
                                r3 += 1;
                            }
                            let mut r4: usize = 0;
                            while r4 < ncols {
                                let sub = f * q[idx(r4, ncols, t)];
                                let nv = q[idx(r4, ncols, j)] - sub;
                                q[idx(r4, ncols, j)] = nv;
                                r4 += 1;
                            }
                            let left = a[idx(t, ncols, j)];
                            if left != 0 {
                                // Move the (smaller) remainder into (t, t).
                                let mut r5: usize = 0;
                                while r5 < nrows {
                                    let tmp = a[idx(r5, ncols, t)];
                                    a[idx(r5, ncols, t)] = a[idx(r5, ncols, j)];
                                    a[idx(r5, ncols, j)] = tmp;
                                    r5 += 1;
                                }
                                let mut c5: usize = 0;
                                while c5 < ncols {
                                    let tmp = q[idx(c5, ncols, t)];
                                    q[idx(c5, ncols, t)] = q[idx(c5, ncols, j)];
                                    q[idx(c5, ncols, j)] = tmp;
                                    c5 += 1;
                                }
                                clean = false;
                            }
                        }
                    }
                    j += 1;
                }

                // 4. Divisibility of the trailing sub-matrix by the pivot.
                let mut bad = nrows;
                if clean {
                    let mut i2: usize = t + 1;
                    while i2 < nrows {
                        let mut j2: usize = t + 1;
                        while j2 < ncols {
                            let v = a[idx(i2, ncols, j2)];
                            let rem = v % pivot;
                            if rem != 0 {
                                if i2 < bad {
                                    bad = i2;
                                }
                            }
                            j2 += 1;
                        }
                        i2 += 1;
                    }
                }

                if clean {
                    if bad != nrows {
                        // `row_t += row_bad`, so that the non-divisible entry is
                        // carried into row `t` and gets cleared or swapped.
                        let mut c: usize = 0;
                        while c < ncols {
                            let s = a[idx(t, ncols, c)] + a[idx(bad, ncols, c)];
                            a[idx(t, ncols, c)] = s;
                            c += 1;
                        }
                        let mut c2: usize = 0;
                        while c2 < nrows {
                            let s = p[idx(t, nrows, c2)] + p[idx(bad, nrows, c2)];
                            p[idx(t, nrows, c2)] = s;
                            c2 += 1;
                        }
                    } else {
                        // Row and column `t` are clean and the pivot divides the
                        // whole trailing sub-matrix: move on to the next pivot.
                        advanced = true;
                        let tn = t + 1;
                        t = tn;
                    }
                }
            }
        }
    }

    // Normalise the sign of the invariant factors: negate row `i` (of `d` and
    // of the transform `p`, which is unit-lower-triangular so its diagonal is
    // untouched).
    let mut i3: usize = 0;
    while i3 < nm {
        let d = a[idx(i3, ncols, i3)];
        if d < 0 {
            let mut c: usize = 0;
            while c < ncols {
                let nv = 0 - a[idx(i3, ncols, c)];
                a[idx(i3, ncols, c)] = nv;
                c += 1;
            }
            let mut c2: usize = 0;
            while c2 < nrows {
                let nv = 0 - p[idx(i3, nrows, c2)];
                p[idx(i3, nrows, c2)] = nv;
                c2 += 1;
            }
        }
        i3 += 1;
    }

    let result = SmithForm { p: p, d: a, q: q };
    result
}
