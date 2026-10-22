//! Two-Way string matching (Crochemore–Perrin, JACM 1991).
//!
//! `two_way_find(t, p)` returns *all* starting positions of the pattern `p` in
//! the text `t`, in increasing order (the empty pattern matches nothing).  The
//! scan runs in `O(|t| + |p|)` time and `O(1)` extra space: the pattern is
//! pre-processed once into a *critical factorization* `p = u v` together with
//! the period of `v`, and each candidate position is then checked by matching
//! the right half `v` left-to-right first and only then the left half `u`
//! right-to-left (the `memory` variable remembers how much of `u` is already
//! known to match).
//!
//! Characters are `usize`-encoded byte values in `0..=255`; no `as` casts are
//! used, so the only numeric model involved is `usize` (→ `Std.Usize`).

/// Order code of the comparison `a` vs `b` in the direction selected by `rev`.
///
/// `0` = "smaller", `1` = "equal", `2` = "larger"; `rev` compares in the
/// opposite order (used for the scan of the reversed word).  Returning a code
/// keeps the two comparisons out of one long `&&`/`||` chain.
fn cmp_code(a: usize, b: usize, rev: bool) -> usize {
    let mut x = a;
    let mut y = b;
    if rev {
        x = b;
        y = a;
    }
    let mut code: usize = 1;
    if x < y {
        code = 0;
    }
    if y < x {
        code = 2;
    }
    code
}

/// Crochemore–Perrin maximal-suffix scan of `w`.
///
/// Returns `(ell, per)` where the maximal suffix of `w` (for the order `<=`,
/// or for the reverse order when `rev` is set) starts at `ell - 1`, and `per`
/// is the period associated with it.  `ell` is therefore already `ms + 1`,
/// i.e. the index of the first character of the right half of the
/// factorization — no signed `ms = -1` is ever needed.
///
/// Loop invariants (all `usize`): `left <= right < n`, `offset < period`,
/// and `w[left + offset]`/`w[right + offset]` are in bounds because
/// `right + offset < n`.
fn maximal_suffix(w: &[usize], rev: bool) -> (usize, usize) {
    let n = w.len();
    let mut left: usize = 0;
    let mut right: usize = 1;
    let mut offset: usize = 0;
    let mut period: usize = 1;
    while right + offset < n {
        let a = w[right + offset];
        let b = w[left + offset];
        let code = cmp_code(a, b, rev);
        if code == 0 {
            // The current extension is smaller: the period is everything so far.
            let r = right + offset + 1;
            right = r;
            offset = 0;
            period = right - left;
        } else if code == 1 {
            // Equal: advance through the repetition of the current period.
            let k1 = offset + 1;
            if k1 == period {
                let r = right + offset + 1;
                right = r;
                offset = 0;
            } else {
                offset = k1;
            }
        } else {
            // The current extension is larger: restart from this position.
            left = right;
            right = right + 1;
            offset = 0;
            period = 1;
        }
    }
    (left, period)
}

/// Critical factorization of a non-empty word `w`.
///
/// Returns `(ell, per)` with `ell = ms + 1`, where `ms` is the *larger* of the
/// two maximal-suffix positions computed by `maximal_suffix` (in the plain
/// order and in the reverse order).  The left half is then `w[..ell]` and the
/// right half is `w[ell..]`.  The returned `per` is a period of the right half
/// and a lower bound on the (global) minimal period of `w`.
fn critical_factorization(w: &[usize]) -> (usize, usize) {
    let (ell0, per0) = maximal_suffix(w, false);
    let (ell1, per1) = maximal_suffix(w, true);
    let mut ell = ell1;
    let mut per = per1;
    if ell0 > ell1 {
        ell = ell0;
        per = per0;
    }
    let r = (ell, per);
    r
}

/// All starting positions of `pattern` in `text`, in increasing order.
///
/// The empty pattern has no occurrence, so the result is empty; the same holds
/// when the pattern is longer than the text.
pub fn two_way_find(text: &[usize], pattern: &[usize]) -> Vec<usize> {
    let n = text.len();
    let m = pattern.len();
    let mut occ: Vec<usize> = Vec::new();
    if m == 0 {
        return occ;
    }
    if m > n {
        return occ;
    }

    let (ell, per0) = critical_factorization(pattern);

    // The period of the right half is exact (it is then the global period of
    // the whole pattern) exactly when the left half `p[..ell]` sits at the
    // start of the right half.  Otherwise the algorithm uses the safe lower
    // bound `max(ell, m - ell) + 1` and drops the memorisation.
    let mut long_p = false;
    let mut k = 0;
    while k < ell {
        let a = pattern[k];
        let b = pattern[per0 + k];
        if a != b {
            long_p = true;
            break;
        }
        k += 1;
    }
    let mut per = per0;
    if long_p {
        let gap = m - ell;
        let mx = if ell > gap { ell } else { gap };
        let p1 = mx + 1;
        per = p1;
    }

    // `memory` counts the characters at the end of the left half that are
    // already known to match at the current position; it is only used when the
    // period is exact, and stays `0` in the long-period case.
    let mut pos = 0;
    let mut memory = 0;
    while pos + m <= n {
        // Phase 1: match the right half `pattern[ell..m]`, left to right.
        let mut start = ell;
        if memory > start {
            start = memory;
        }
        let mut i = start;
        let mut right_ok = true;
        while i < m {
            let a = pattern[i];
            let b = text[pos + i];
            if a != b {
                right_ok = false;
                break;
            }
            i += 1;
        }
        if !right_ok {
            // `i` is the first mismatch inside the right half.
            let shift = i + 1 - ell;
            pos += shift;
            memory = 0;
        } else {
            // Phase 2: match the left half `pattern[..ell]`, right to left.
            let mut j = ell;
            let mut left_ok = true;
            while j > memory {
                j -= 1;
                let a = pattern[j];
                let b = text[pos + j];
                if a != b {
                    left_ok = false;
                    break;
                }
            }
            if !left_ok {
                pos += per;
                if !long_p {
                    memory = m - per;
                }
            } else {
                occ.push(pos);
                // Shifting by the period (rather than by `m`) reports the
                // overlapping occurrences as well.
                pos += per;
                if !long_p {
                    memory = m - per;
                }
            }
        }
    }
    occ
}
