//! Tests for the Smith normal form.
//!
//! The expected invariant factors were computed independently from the
//! *determinantal divisors*: `d_k = Δ_k / Δ_{k-1}`, where `Δ_k` is the gcd of
//! all `k × k` minors (`Δ_0 = 1`).

/// Flat index of `(row, col)` in a row-major matrix with `col` columns.
fn fidx(row: usize, cols: usize, col: usize) -> usize {
    let t = row * cols;
    let r = t + col;
    r
}

/// Exact determinant by fraction-free (Bareiss) elimination with row pivoting.
fn mat_det(a: &[i64], n: usize) -> i64 {
    if n == 0 {
        return 1;
    }
    let mut m: Vec<i64> = a.to_vec();
    let mut prev: i64 = 1;
    let mut sign: i64 = 1;
    let mut k: usize = 0;
    while k + 1 < n {
        if m[fidx(k, n, k)] == 0 {
            // Bring a non-zero entry of column k up to the diagonal.
            let mut found = n;
            let mut r: usize = k + 1;
            while r < n {
                let v = m[fidx(r, n, k)];
                if v != 0 {
                    if r < found {
                        found = r;
                    }
                }
                r += 1;
            }
            if found == n {
                return 0;
            }
            let mut c: usize = 0;
            while c < n {
                let tmp = m[fidx(k, n, c)];
                m[fidx(k, n, c)] = m[fidx(found, n, c)];
                m[fidx(found, n, c)] = tmp;
                c += 1;
            }
            let ns = 0 - sign;
            sign = ns;
        }
        let pivot = m[fidx(k, n, k)];
        let mut i: usize = k + 1;
        while i < n {
            let mut j: usize = k + 1;
            while j < n {
                let a_ij = m[fidx(i, n, j)];
                let a_ik = m[fidx(i, n, k)];
                let a_kj = m[fidx(k, n, j)];
                let num = a_ij * pivot - a_ik * a_kj;
                let quo = num / prev;
                m[fidx(i, n, j)] = quo;
                j += 1;
            }
            i += 1;
        }
        prev = pivot;
        k += 1;
    }
    let last = m[fidx(n - 1, n, n - 1)];
    let out = sign * last;
    out
}

/// Element-wise equality of two vectors.
fn veq(a: &[i64], b: &[i64]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut same = true;
    let mut i: usize = 0;
    while i < a.len() {
        if a[i] != b[i] {
            same = false;
        }
        i += 1;
    }
    same
}

/// Product of the `r × k` matrix `a` with the `k × c` matrix `b` (row-major).
fn mat_mul(a: &[i64], b: &[i64], r: usize, k: usize, c: usize) -> Vec<i64> {
    let mut out = vec![0i64; r * c];
    let mut i: usize = 0;
    while i < r {
        let mut j: usize = 0;
        while j < c {
            let mut sum: i64 = 0;
            let mut t: usize = 0;
            while t < k {
                let p = a[fidx(i, k, t)] * b[fidx(t, c, j)];
                sum = sum + p;
                t += 1;
            }
            out[fidx(i, c, j)] = sum;
            j += 1;
        }
        i += 1;
    }
    out
}

/// Every entry off the main diagonal is zero.
fn is_diagonal(d: &[i64], nrows: usize, ncols: usize) -> bool {
    let mut ok = true;
    let mut i: usize = 0;
    while i < nrows {
        let mut j: usize = 0;
        while j < ncols {
            if i != j {
                if d[fidx(i, ncols, j)] != 0 {
                    ok = false;
                }
            }
            j += 1;
        }
        i += 1;
    }
    ok
}

/// The diagonal is non-negative and forms a divisibility chain `d₀ | d₁ | …`.
fn chain_ok(d: &[i64], nrows: usize, ncols: usize) -> bool {
    let nm = if nrows < ncols { nrows } else { ncols };
    let mut ok = true;
    let mut i: usize = 0;
    while i < nm {
        let di = d[fidx(i, ncols, i)];
        if di < 0 {
            ok = false;
        }
        if i + 1 < nm {
            let dj = d[fidx(i + 1, ncols, i + 1)];
            if di == 0 {
                if dj != 0 {
                    ok = false;
                }
            } else {
                let quo = dj / di;
                let back = quo * di;
                if back != dj {
                    ok = false;
                }
            }
        }
        i += 1;
    }
    ok
}

/// A determinant of `±1` (unimodularity), tested without `||` on comparisons.
fn unimodular(det: i64) -> bool {
    let mut ok = false;
    if det == 1 {
        ok = true;
    }
    if det == 0 - 1 {
        ok = true;
    }
    ok
}

/// Full check of one Smith normal form computation.
fn check_smith(a: &[i64], nrows: usize, ncols: usize, expected_d: &[i64]) {
    let res = matrix_smith_normal_form::smith_normal_form(a, nrows, ncols);

    // 1. The invariant factors are the expected ones.
    assert!(veq(&res.d, expected_d));

    // 2. d = p · a · q.
    let pa = mat_mul(&res.p, a, nrows, nrows, ncols);
    let paq = mat_mul(&pa, &res.q, nrows, ncols, ncols);
    assert!(veq(&paq, &res.d));

    // 3. d is diagonal, non-negative, with a divisibility chain.
    assert!(is_diagonal(&res.d, nrows, ncols));
    assert!(chain_ok(&res.d, nrows, ncols));

    // 4. p and q are unimodular.
    assert!(unimodular(mat_det(&res.p, nrows)));
    assert!(unimodular(mat_det(&res.q, ncols)));
}

/// 0 × 0: nothing to do.
#[test]
fn test_smith_empty() {
    let a: Vec<i64> = Vec::new();
    let d: Vec<i64> = Vec::new();
    check_smith(&a, 0, 0, &d);
}

/// 1 × 1: the single invariant factor is the absolute value.
#[test]
fn test_smith_1x1() {
    let a = vec![5i64];
    let d = vec![5i64];
    check_smith(&a, 1, 1, &d);

    let b = vec![-3i64];
    let e = vec![3i64];
    check_smith(&b, 1, 1, &e);
}

/// The zero matrix has all invariant factors zero.
#[test]
fn test_smith_zero() {
    let a = vec![0i64; 6];
    let d = vec![0i64; 6];
    check_smith(&a, 2, 3, &d);
}

/// [[2, 4], [6, 8]]: d₁ = gcd(2,4,6,8) = 2, d₂ = 8/2 = 4.
#[test]
fn test_smith_2x2_full() {
    #[rustfmt::skip]
    let a = vec![
        2i64, 4,
        6,    8,
    ];
    #[rustfmt::skip]
    let d = vec![
        2i64, 0,
        0,    4,
    ];
    check_smith(&a, 2, 2, &d);
}

/// [[2, 4], [4, 8]]: rank 1 — the second invariant factor is 0.
#[test]
fn test_smith_2x2_rank_one() {
    #[rustfmt::skip]
    let a = vec![
        2i64, 4,
        4,    8,
    ];
    #[rustfmt::skip]
    let d = vec![
        2i64, 0,
        0,    0,
    ];
    check_smith(&a, 2, 2, &d);
}

/// [[2, 0], [0, 3]] is already diagonal but *not* in Smith form:
/// the coprime entries merge into d₁ = 1, d₂ = 6.
#[test]
fn test_smith_2x2_diagonal_coprime() {
    #[rustfmt::skip]
    let a = vec![
        2i64, 0,
        0,    3,
    ];
    #[rustfmt::skip]
    let d = vec![
        1i64, 0,
        0,    6,
    ];
    check_smith(&a, 2, 2, &d);
}

/// [[-2, 4], [6, 8]]: the sign is normalised away.
#[test]
fn test_smith_2x2_signed() {
    #[rustfmt::skip]
    let a = vec![
        -2i64, 4,
         6,    8,
    ];
    #[rustfmt::skip]
    let d = vec![
        2i64, 0,
        0,   20,
    ];
    check_smith(&a, 2, 2, &d);
}

/// 2 × 3: [[1, 2, 3], [4, 5, 6]] → diag(1, 3).
#[test]
fn test_smith_2x3() {
    #[rustfmt::skip]
    let a = vec![
        1i64, 2, 3,
        4,    5, 6,
    ];
    #[rustfmt::skip]
    let d = vec![
        1i64, 0, 0,
        0,    3, 0,
    ];
    check_smith(&a, 2, 3, &d);
}

/// 3 × 2: [[1, 2], [3, 4], [5, 6]] → diag(1, 2).
#[test]
fn test_smith_3x2() {
    #[rustfmt::skip]
    let a = vec![
        1i64, 2,
        3,    4,
        5,    6,
    ];
    #[rustfmt::skip]
    let d = vec![
        1i64, 0,
        0,    2,
        0,    0,
    ];
    check_smith(&a, 3, 2, &d);
}

/// 3 × 3 with determinant 3: [[1,2,3],[4,5,6],[7,8,10]] → diag(1, 1, 3).
#[test]
fn test_smith_3x3() {
    #[rustfmt::skip]
    let a = vec![
        1i64, 2, 3,
        4,    5, 6,
        7,    8, 10,
    ];
    #[rustfmt::skip]
    let d = vec![
        1i64, 0, 0,
        0,    1, 0,
        0,    0, 3,
    ];
    check_smith(&a, 3, 3, &d);
}

/// 3 × 3 with negative entries: → diag(1, 1, 360).
#[test]
fn test_smith_3x3_signed() {
    #[rustfmt::skip]
    let a = vec![
        -1i64,  2,  3,
         4,    -5,  6,
         7,     8, -9,
    ];
    #[rustfmt::skip]
    let d = vec![
        1i64, 0,   0,
        0,    1,   0,
        0,    0, 360,
    ];
    check_smith(&a, 3, 3, &d);
}

/// Symmetric matrix whose Smith form needs the divisibility fix-up:
/// [[2,4,4],[4,8,4],[4,4,8]] → diag(2, 4, 4).
#[test]
fn test_smith_3x3_symmetric() {
    #[rustfmt::skip]
    let a = vec![
        2i64, 4, 4,
        4,    8, 4,
        4,    4, 8,
    ];
    #[rustfmt::skip]
    let d = vec![
        2i64, 0, 0,
        0,    4, 0,
        0,    0, 4,
    ];
    check_smith(&a, 3, 3, &d);
}

/// diag(6, 10, 15) — pairwise coprime entries collapse to diag(1, 30, 30).
#[test]
fn test_smith_3x3_diagonal() {
    #[rustfmt::skip]
    let a = vec![
        6i64, 0,   0,
        0,    10,  0,
        0,    0,   15,
    ];
    #[rustfmt::skip]
    let d = vec![
        1i64, 0,  0,
        0,    30, 0,
        0,    0,  30,
    ];
    check_smith(&a, 3, 3, &d);
}

/// A rank-2 4 × 4 matrix: the 1..16 matrix has Smith form diag(1, 4, 0, 0).
#[test]
fn test_smith_4x4_rank_two() {
    #[rustfmt::skip]
    let a = vec![
         1i64,  2,  3,  4,
         5,     6,  7,  8,
         9,    10, 11, 12,
        13,    14, 15, 16,
    ];
    #[rustfmt::skip]
    let d = vec![
        1i64, 0, 0, 0,
        0,    4, 0, 0,
        0,    0, 0, 0,
        0,    0, 0, 0,
    ];
    check_smith(&a, 4, 4, &d);
}
