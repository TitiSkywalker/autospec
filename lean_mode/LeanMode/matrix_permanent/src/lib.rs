//! Permanent of an n×n integer matrix via Ryser's inclusion–exclusion formula.
//!
//! The permanent is the "sign-free determinant"
//!
//! ```text
//!   per(A) = Σ_{σ ∈ S_n} Π_{i} A[i][σ(i)]
//! ```
//!
//! and is #P-complete to compute; the O(n!)-enumeration above is useless in
//! practice.  Ryser's formula evaluates it in `O(2ⁿ·n²)` by summing over
//! *subsets* of the columns instead of permutations:
//!
//! ```text
//!   per(A) = (-1)ⁿ · Σ_{S ⊆ {0,…,n-1}} (-1)^{|S|} · Π_i (Σ_{j ∈ S} A[i][j])
//! ```
//!
//! The subsets are enumerated with an explicit binary counter over a flag
//! array (no bit-fiddling on `usize`), and the two-dimensional sum is grouped
//! by `|S|` so that each alternating sign is applied once per subset size
//! rather than once per subset.
//!
//! All matrices are stored in row-major order; `i64` is used so that the true
//! (possibly negative) value is represented exactly.

/// Flat index of `(row, col)` in a row-major `n × n` matrix.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Permanent of the `n × n` matrix `matrix` (row-major, `n²` entries).
///
/// Uses Ryser's formula, see the module documentation.  Returns `1` for
/// `n = 0` (the permanent of the empty matrix is the empty product).
pub fn permanent(matrix: &[i64], n: usize) -> i64 {
    // acc[k] = Σ over the k-element subsets S of Π_i (Σ_{j ∈ S} A[i][j]).
    let mut acc = vec![0i64; n + 1];

    // Number of subsets, 2ⁿ.
    let mut subsets: usize = 1;
    let mut t: usize = 0;
    while t < n {
        subsets = subsets * 2;
        t += 1;
    }

    // `sel[j] == 1` iff column j is in the current subset; `count = |S|`.
    let mut sel = vec![0i64; n];
    let mut count: usize = 0;
    let mut step: usize = 0;
    while step < subsets {
        // Π_i (Σ_{j ∈ S} A[i][j]) for the current subset.
        let mut prod: i64 = 1;
        let mut i: usize = 0;
        while i < n {
            let mut rowsum: i64 = 0;
            let mut j: usize = 0;
            while j < n {
                if sel[j] == 1 {
                    let v = matrix[idx(i, n, j)];
                    let s = rowsum + v;
                    rowsum = s;
                }
                j += 1;
            }
            let p = prod * rowsum;
            prod = p;
            i += 1;
        }
        let old = acc[count];
        let new = old + prod;
        acc[count] = new;

        // Binary-counter step to the next subset: clear the trailing ones and
        // set the first zero.
        let mut j: usize = 0;
        let mut carry = true;
        while j < n {
            if carry {
                if sel[j] == 1 {
                    sel[j] = 0;
                    let c = count - 1;
                    count = c;
                } else {
                    sel[j] = 1;
                    let c = count + 1;
                    count = c;
                    carry = false;
                }
            }
            j += 1;
        }

        step += 1;
    }

    // per(A) = Σ_k (-1)^(n-k) · acc[k].
    let mut total: i64 = 0;
    let mut k: usize = 0;
    while k <= n {
        let diff = n - k;
        let half = diff / 2;
        let twice = half * 2;
        if twice == diff {
            let s = total + acc[k];
            total = s;
        } else {
            let s = total - acc[k];
            total = s;
        }
        k += 1;
    }
    total
}
