//! Subset convolution by ranked zeta/Möbius transforms (SOS DP).
//!
//! Given two arrays `a`, `b` of length `2ⁿ` that are indexed by subsets of an
//! `n`-element set, the **subset convolution** is
//!
//! ```text
//! c[S] = Σ_{T ⊆ S} a[T] · b[S \ T]
//! ```
//!
//! which naively costs `O(3ⁿ)`.  The ranked-transform algorithm costs
//! `O(2ⁿ n²)`:
//!
//! 1. spread `a` over `n + 1` *rank* layers: `f[k][S] = a[S]` when
//!    `popcount(S) = k` and `0` otherwise (same for `b`);
//! 2. zeta transform each layer over the subsets (`f[k][S] += f[k][S \ {i}]`);
//!    this turns disjointness into a convolution in the rank;
//! 3. multiply pointwise, summing the rank splits:
//!    `h[r][S] = Σ_{j ≤ r} f[j][S] · g[r - j][S]`;
//! 4. Möbius transform each layer back (`h[r][S] -= h[r][S \ {i}]`);
//! 5. read the answer off the diagonal: `c[S] = h[popcount(S)][S]`.
//!
//! The rank layers keep the two factors of a product disjoint, which is what
//! makes the (otherwise wrong) pointwise product of the zeta transforms
//! meaningful.

/// Number of set bits of `x`.
fn popcount(x: usize) -> usize {
    let mut c = x;
    let mut count = 0;
    while c > 0 {
        let bit = c - (c / 2) * 2;
        count += bit;
        c = c / 2;
    }
    count
}

/// Subset convolution of `a` and `b`, both of length `2ⁿ`.
pub fn subset_convolution(a: &[usize], b: &[usize]) -> Vec<usize> {
    let len = a.len();
    // n = log2(len)
    let mut n = 0usize;
    let mut sz = 1usize;
    while sz < len {
        sz = sz * 2;
        n += 1;
    }
    let size = 1usize << n;
    let stride = size;

    // 1. rank layers
    let mut f = vec![0usize; (n + 1) * stride];
    let mut g = vec![0usize; (n + 1) * stride];
    let mut s = 0;
    while s < size {
        let pc = popcount(s);
        let idx = pc * stride + s;
        f[idx] = a[s];
        g[idx] = b[s];
        s += 1;
    }

    // 2. zeta transform (subset sums) of every rank layer
    let mut rank = 0;
    while rank <= n {
        let base = rank * stride;
        let mut i = 0;
        while i < n {
            let bit = 1usize << i;
            let mut m = 0;
            while m < size {
                let has = m & bit;
                if has != 0 {
                    let j = m - bit;
                    let x = f[base + m] + f[base + j];
                    f[base + m] = x;
                    let y = g[base + m] + g[base + j];
                    g[base + m] = y;
                }
                m += 1;
            }
            i += 1;
        }
        rank += 1;
    }

    // 3. pointwise product over the rank splits
    let mut h = vec![0usize; (n + 1) * stride];
    let mut r = 0;
    while r <= n {
        let mut t = 0;
        while t < size {
            let mut acc = 0usize;
            let mut j = 0;
            while j <= r {
                let p = f[j * stride + t] * g[(r - j) * stride + t];
                acc += p;
                j += 1;
            }
            let idx = r * stride + t;
            h[idx] = acc;
            t += 1;
        }
        r += 1;
    }

    // 4. Möbius transform (subset differences) of every rank layer
    let mut rank2 = 0;
    while rank2 <= n {
        let base = rank2 * stride;
        let mut i = 0;
        while i < n {
            let bit = 1usize << i;
            let mut m = 0;
            while m < size {
                let has = m & bit;
                if has != 0 {
                    let j = m - bit;
                    let x = h[base + m] - h[base + j];
                    h[base + m] = x;
                }
                m += 1;
            }
            i += 1;
        }
        rank2 += 1;
    }

    // 5. read the answer off the diagonal
    let mut c = vec![0usize; size];
    let mut t2 = 0;
    while t2 < size {
        let pc = popcount(t2);
        let idx = pc * stride + t2;
        c[t2] = h[idx];
        t2 += 1;
    }
    c
}
