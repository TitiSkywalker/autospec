//! Decision-monotone (1D1D) dynamic programming.
//!
//! For a **non-decreasing** array `p` of positions, define
//!
//! ```text
//! dp[0] = 0
//! dp[i] = min over 0 <= j < i of ( dp[j] + (p[i] - p[j])^2 )
//! ```
//!
//! The transition cost `(p[i] - p[j])^2` satisfies the quadrangle inequality
//! (for `a <= b <= c <= d`, `w(a,c) + w(b,d) <= w(a,d) + w(b,c)`), which makes
//! the optimal decision point `opt[i]` non-decreasing in `i`.  We provide
//!
//! * `dp_naive`     — the O(n^2) reference double loop, and
//! * `dp_optimized` — an O(n log n) monotone-queue (1D1D) optimization: a deque
//!   of `(decision, first target)` buckets, where the crossover between two
//!   decisions is found by binary search.
//!
//! Both return `dp[n-1]` where `n = p.len()` (and `0` for the empty input).
//! Both `assert!` that `p` is non-decreasing, so an `ok` result *implies* the
//! hypothesis that the specification relies on.

/// `|a - b|` for `usize`, written to guard against underflow.
fn udiff(a: usize, b: usize) -> usize {
    let d = if a >= b { a - b } else { b - a };
    d
}

/// The transition cost `(px - pj)^2` of moving from decision `pj` to target `px`.
fn sq_cost(px: usize, pj: usize) -> usize {
    let d = udiff(px, pj);
    let dd = d * d;
    dd
}

/// `dp[j] + (p[x] - p[j])^2`: the value of using decision `j` for target `x`.
fn cand_cost(dp: &[usize], p: &[usize], j: usize, x: usize) -> usize {
    let pj = p[j];
    let px = p[x];
    let c = sq_cost(px, pj);
    let dj = dp[j];
    let v = dj + c;
    v
}

/// Smallest `x` in `[lo, hi]` with `cand_cost dp p i x <= cand_cost dp p b x`,
/// assuming the predicate already holds at `hi`.  For a non-decreasing `p` the
/// difference of the two candidate functions is non-increasing in `x` (each pair
/// of candidates crosses exactly once), so binary search is valid here.
fn crossover(dp: &[usize], p: &[usize], i: usize, b: usize, lo: usize, hi: usize) -> usize {
    let mut l = lo;
    let mut r = hi;
    while l < r {
        let mid = (l + r) / 2;
        let vi = cand_cost(dp, p, i, mid);
        let vb = cand_cost(dp, p, b, mid);
        if vi <= vb {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    l
}

/// Naive O(n^2) evaluation of the recurrence; returns `dp[n-1]`.
pub fn dp_naive(p: &[usize]) -> usize {
    let n = p.len();
    let mut k = 0;
    while k + 1 < n {
        assert!(p[k] <= p[k + 1]);
        k += 1;
    }
    if n == 0 {
        return 0;
    }
    let mut dp: Vec<usize> = vec![0usize; n];
    let mut i = 1;
    while i < n {
        let pi = p[i];
        let p0 = p[0];
        let c0 = sq_cost(pi, p0);
        let d0 = dp[0];
        let mut best = d0 + c0;
        let mut j = 1;
        while j < i {
            let pj = p[j];
            let c = sq_cost(pi, pj);
            let dj = dp[j];
            let cand = dj + c;
            if cand < best {
                best = cand;
            }
            j += 1;
        }
        dp[i] = best;
        i += 1;
    }
    dp[n - 1]
}

/// Monotone-queue O(n log n) evaluation of the recurrence; returns `dp[n-1]`.
///
/// The deque stores, for each decision `j`, the smallest target index `s` from
/// which `j` is optimal.  Those intervals partition `[front_start, n-1]`, so the
/// front bucket is optimal for the current target and the crossover of a new
/// decision against the back can be computed by binary search.
pub fn dp_optimized(p: &[usize]) -> usize {
    let n = p.len();
    let mut k = 0;
    while k + 1 < n {
        assert!(p[k] <= p[k + 1]);
        k += 1;
    }
    if n == 0 {
        return 0;
    }
    let mut dp: Vec<usize> = vec![0usize; n];
    let cap = n + 1;
    let mut dqj: Vec<usize> = vec![0usize; cap];
    let mut dqs: Vec<usize> = vec![0usize; cap];
    let mut h: usize = 0;
    dqs[0] = 1;
    let mut len: usize = 1;

    let mut i = 1;
    while i < n {
        // (1) Advance the front so its bucket covers target `i`.
        let mut nxt = h + 1;
        let mut s: usize = n;
        if nxt < len {
            s = dqs[nxt];
        }
        while s <= i {
            h = nxt;
            nxt = h + 1;
            s = n;
            if nxt < len {
                s = dqs[nxt];
            }
        }

        // (2) The front decision is optimal for `i`.
        let jb = dqj[h];
        let v = cand_cost(&dp, p, jb, i);
        dp[i] = v;

        // (3) Insert `i` as a new decision for the (strictly later) targets.
        if i + 1 < n {
            // Pop back decisions that `i` dominates.
            let mut popping: bool = true;
            while popping {
                let empty = len <= h;
                if empty {
                    popping = false;
                } else {
                    let bj = dqj[len - 1];
                    let bs = dqs[len - 1];
                    let i1 = i + 1;
                    let t = if bs > i1 { bs } else { i1 };
                    let vi = cand_cost(&dp, p, i, t);
                    let vb = cand_cost(&dp, p, bj, t);
                    if vi <= vb {
                        let nl = len - 1;
                        len = nl;
                    } else {
                        popping = false;
                    }
                }
            }
            let empty2 = len <= h;
            if empty2 {
                dqj[len] = i;
                let st = i + 1;
                dqs[len] = st;
                let nl = len + 1;
                len = nl;
            } else {
                let bj = dqj[len - 1];
                let hi = n - 1;
                let vi2 = cand_cost(&dp, p, i, hi);
                let vb2 = cand_cost(&dp, p, bj, hi);
                if vi2 <= vb2 {
                    let bs = dqs[len - 1];
                    let i1 = i + 1;
                    let lo = if bs > i1 { bs } else { i1 };
                    let x0 = crossover(&dp, p, i, bj, lo + 1, hi);
                    dqj[len] = i;
                    dqs[len] = x0;
                    let nl = len + 1;
                    len = nl;
                }
            }
        }
        i += 1;
    }
    dp[n - 1]
}
