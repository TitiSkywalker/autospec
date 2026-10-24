//! Segment Tree Beats (Ji driver segment tree).
//!
//! Supports range `chmin` (`a[i] = min(a[i], v)`) plus range-max and range-sum
//! queries, using the classic "second maximum" lazy propagation: a `chmin` that
//! only touches the segment's maximum values is applied lazily in O(1); a
//! deeper `chmin` recurses.  Each node stores `mx` (max), `se` (strict second
//! max; `-1` when all values equal `mx`), `cn` (count of max), `sum`, and a
//! lazy `tag`.  All array values are assumed `> -1`.

/// The segment-tree-beats node arrays (1-indexed; `n` = number of leaves).
pub struct SegBeats {
    pub mx: Vec<i64>,
    pub se: Vec<i64>,
    pub cn: Vec<i64>,
    pub sum: Vec<i64>,
    pub tag: Vec<i64>,
    pub n: usize,
}

// Comparison helpers (hand-expanded, returning `bool` so the translator sees
// them as booleans rather than Lean `Prop`s).
fn le(a: usize, b: usize) -> bool {
    if a <= b { true } else { false }
}
fn lt(a: usize, b: usize) -> bool {
    if a < b { true } else { false }
}
fn ile(a: i64, b: i64) -> bool {
    if a <= b { true } else { false }
}
fn imax(a: i64, b: i64) -> i64 {
    if a > b { a } else { b }
}

/// Build a segment tree beats over `a`.
pub fn segbeats_build(a: &[i64]) -> SegBeats {
    let n = a.len();
    let size = n * 4 + 4;
    let st = SegBeats {
        mx: vec![0; size],
        se: vec![0; size],
        cn: vec![0; size],
        sum: vec![0; size],
        tag: vec![0; size],
        n,
    };
    build_rec(st, a, 1, 0, n - 1)
}

fn build_rec(st: SegBeats, a: &[i64], u: usize, l: usize, r: usize) -> SegBeats {
    let mut s = st;
    let neg = -1;
    s.tag[u] = neg;
    if l == r {
        let x = a[l];
        s.mx[u] = x;
        s.se[u] = neg;
        s.cn[u] = 1;
        s.sum[u] = x;
        s
    } else {
        let mid = (l + r) / 2;
        let ls = u * 2;
        let rs = u * 2 + 1;
        let s = build_rec(s, a, ls, l, mid);
        let s = build_rec(s, a, rs, mid + 1, r);
        pushup(s, u)
    }
}

/// Recompute `u`'s satellite info from its two children.
fn pushup(st: SegBeats, u: usize) -> SegBeats {
    let mut s = st;
    let ls = u * 2;
    let rs = u * 2 + 1;
    let sl = s.sum[ls];
    let sr = s.sum[rs];
    let t = sl + sr;
    s.sum[u] = t;
    let ml = s.mx[ls];
    let mr = s.mx[rs];
    if ml == mr {
        s.mx[u] = ml;
        let se_l = s.se[ls];
        let se_r = s.se[rs];
        let se_t = imax(se_l, se_r);
        s.se[u] = se_t;
        let cl = s.cn[ls];
        let cr = s.cn[rs];
        let ct = cl + cr;
        s.cn[u] = ct;
    } else if ml > mr {
        s.mx[u] = ml;
        let se_l = s.se[ls];
        let se_t = imax(se_l, mr);
        s.se[u] = se_t;
        s.cn[u] = s.cn[ls];
    } else {
        s.mx[u] = mr;
        let se_r = s.se[rs];
        let se_t = imax(ml, se_r);
        s.se[u] = se_t;
        s.cn[u] = s.cn[rs];
    }
    s
}

/// Apply a `chmin(v)` tag to node `u` (no-op if `mx[u] ≤ v`).
fn pushtag(st: SegBeats, u: usize, v: i64) -> SegBeats {
    let mut s = st;
    let mu = s.mx[u];
    if ile(mu, v) {
        s
    } else {
        let c = s.cn[u];
        let d = v - mu;
        let delta = d * c;
        let ns = s.sum[u] + delta;
        s.sum[u] = ns;
        s.mx[u] = v;
        s.tag[u] = v;
        s
    }
}

/// Push node `u`'s lazy tag down to its children.
fn pushdown(st: SegBeats, u: usize) -> SegBeats {
    let mut s = st;
    let tg = s.tag[u];
    if tg == -1 {
        s
    } else {
        let ls = u * 2;
        let rs = u * 2 + 1;
        s = pushtag(s, ls, tg);
        s = pushtag(s, rs, tg);
        let neg = -1;
        s.tag[u] = neg;
        s
    }
}

fn modify_min(st: SegBeats, L: usize, R: usize, v: i64, u: usize, l: usize, r: usize) -> SegBeats {
    let mu = st.mx[u];
    if ile(mu, v) {
        st
    } else {
        let seu = st.se[u];
        let fully = le(L, l) && le(r, R);
        let do_tag = fully && seu < v;
        if do_tag {
            pushtag(st, u, v)
        } else {
            let mid = (l + r) / 2;
            let ls = u * 2;
            let rs = u * 2 + 1;
            let st = pushdown(st, u);
            let st = if le(L, mid) { modify_min(st, L, R, v, ls, l, mid) } else { st };
            let st = if lt(mid, R) { modify_min(st, L, R, v, rs, mid + 1, r) } else { st };
            pushup(st, u)
        }
    }
}

fn query_max_rec(st: SegBeats, L: usize, R: usize, u: usize, l: usize, r: usize) -> (SegBeats, i64) {
    if le(L, l) && le(r, R) {
        let m = st.mx[u];
        (st, m)
    } else {
        let mid = (l + r) / 2;
        let ls = u * 2;
        let rs = u * 2 + 1;
        let st = pushdown(st, u);
        if le(L, mid) {
            let (st, left) = query_max_rec(st, L, R, ls, l, mid);
            if lt(mid, R) {
                let (st, right) = query_max_rec(st, L, R, rs, mid + 1, r);
                let m = imax(left, right);
                (st, m)
            } else {
                (st, left)
            }
        } else {
            let (st, right) = query_max_rec(st, L, R, rs, mid + 1, r);
            (st, right)
        }
    }
}

fn query_sum_rec(st: SegBeats, L: usize, R: usize, u: usize, l: usize, r: usize) -> (SegBeats, i64) {
    if le(L, l) && le(r, R) {
        let sm = st.sum[u];
        (st, sm)
    } else {
        let mid = (l + r) / 2;
        let ls = u * 2;
        let rs = u * 2 + 1;
        let st = pushdown(st, u);
        if le(L, mid) {
            let (st, left) = query_sum_rec(st, L, R, ls, l, mid);
            if lt(mid, R) {
                let (st, right) = query_sum_rec(st, L, R, rs, mid + 1, r);
                let t = left + right;
                (st, t)
            } else {
                (st, left)
            }
        } else {
            let (st, right) = query_sum_rec(st, L, R, rs, mid + 1, r);
            (st, right)
        }
    }
}

/// Range chmin: `a[i] = min(a[i], v)` for all `L ≤ i ≤ R`.
pub fn segbeats_chmin(st: SegBeats, L: usize, R: usize, v: i64) -> SegBeats {
    let n = st.n;
    modify_min(st, L, R, v, 1, 0, n - 1)
}

/// Maximum of `a[L..=R]`.
pub fn segbeats_query_max(st: SegBeats, L: usize, R: usize) -> (SegBeats, i64) {
    let n = st.n;
    query_max_rec(st, L, R, 1, 0, n - 1)
}

/// Sum of `a[L..=R]`.
pub fn segbeats_query_sum(st: SegBeats, L: usize, R: usize) -> (SegBeats, i64) {
    let n = st.n;
    query_sum_rec(st, L, R, 1, 0, n - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brute_query_max(a: &Vec<i64>, l: usize, r: usize) -> i64 {
        let mut m = a[l];
        let mut i = l + 1;
        while i <= r {
            if a[i] > m {
                m = a[i];
            }
            i += 1;
        }
        m
    }

    fn brute_query_sum(a: &Vec<i64>, l: usize, r: usize) -> i64 {
        let mut s = 0;
        let mut i = l;
        while i <= r {
            s = s + a[i];
            i += 1;
        }
        s
    }

    fn brute_chmin(a: &mut Vec<i64>, l: usize, r: usize, v: i64) {
        let mut i = l;
        while i <= r {
            if a[i] > v {
                a[i] = v;
            }
            i += 1;
        }
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *state
    }

    #[test]
    fn basic_chmin_max_sum() {
        let arr = vec![1, 5, 3, 9, 2, 7, 4, 8];
        let st = segbeats_build(&arr);
        let st = segbeats_chmin(st, 1, 5, 4);
        let (st, mx) = segbeats_query_max(st, 0, 7);
        assert_eq!(mx, 8);
        let (st, mx2) = segbeats_query_max(st, 1, 5);
        assert_eq!(mx2, 4);
        let (_st, sm) = segbeats_query_sum(st, 1, 5);
        // after chmin(1..=5, 4): [1,4,3,4,2,4,4,8] → sum 1..5 = 4+3+4+2+4 = 17
        assert_eq!(sm, 17);
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 42;
        let n: usize = 64;
        let mut arr: Vec<i64> = Vec::new();
        let mut i: usize = 0;
        while i < n {
            let r = lcg(&mut state);
            arr.push(((r >> 4) % 50) as i64 + 1);
            i += 1;
        }
        let mut refarr = arr.clone();
        let mut st = segbeats_build(&arr);
        let mut ops: usize = 0;
        while ops < 5000 {
            let r = lcg(&mut state);
            let l = ((r >> 8) % n as u64) as usize;
            let rr = ((r >> 16) % n as u64) as usize;
            let lo = if l < rr { l } else { rr };
            let hi = if l < rr { rr } else { l };
            let v = ((r >> 24) % 60) as i64 + 1;
            if (r & 1) == 0 {
                st = segbeats_chmin(st, lo, hi, v);
                brute_chmin(&mut refarr, lo, hi, v);
            } else {
                let (st1, m) = segbeats_query_max(st, lo, hi);
                st = st1;
                assert_eq!(m, brute_query_max(&refarr, lo, hi), "max op {} range [{},{}]", ops, lo, hi);
                let (st2, s) = segbeats_query_sum(st, lo, hi);
                st = st2;
                assert_eq!(s, brute_query_sum(&refarr, lo, hi), "sum op {} range [{},{}] refarr {:?}", ops, lo, hi, refarr);
            }
            ops += 1;
        }
    }
}
