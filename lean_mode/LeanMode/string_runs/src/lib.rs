//! Runs (maximal periodicities) of a string, found in O(n²).
//!
//! Characters are `usize`-encoded byte values in `0..=255`.  No `as` casts are
//! used, so the only numeric model involved is `usize` (→ `Std.Usize`).

/// A run: the half-open interval `[start, end)` has minimal period `period`.
pub struct Run {
    pub start: usize,
    pub end: usize,
    pub period: usize,
}

/// Boolean disjunction folded through a call (avoids `||` chains).
fn or2(a: bool, b: bool) -> bool {
    if a {
        true
    } else {
        b
    }
}

/// The prefix function (border array) of `s[from..from + m)`.
///
/// `pi[j]` is the length of the longest proper border of the prefix
/// `s[from..from + j + 1)`.  The scan uses `usize` parameters only.
fn prefix_pi(s: &[usize], from: usize, m: usize) -> Vec<usize> {
    let mut pi: Vec<usize> = vec![0usize; m];
    let mut i: usize = 1;
    while i < m {
        let mut j = pi[i - 1];
        let mut back = true;
        while back {
            if j > 0 {
                let a = s[from + i];
                let b = s[from + j];
                if a != b {
                    j = pi[j - 1];
                } else {
                    back = false;
                }
            } else {
                back = false;
            }
        }
        let a = s[from + i];
        let b = s[from + j];
        let mut inc: usize = 0;
        if a == b {
            inc = 1;
        }
        let v = j + inc;
        pi[i] = v;
        i += 1;
    }
    pi
}

/// The index of the run `(start, end, period)` inside `out`, or `out.len()`
/// when that run is not present yet.
///
/// The "minimum index with a property, sentinel = length" scan keeps Aeneas's
/// loop analysis happy (a bare `found` flag does not).
fn dup_index(out: &[Run], start: usize, end: usize, period: usize) -> usize {
    let m = out.len();
    let mut res: usize = m;
    let mut e: usize = 0;
    while e < m {
        let es = out[e].start;
        let ee = out[e].end;
        let ep = out[e].period;
        let d = or2(or2(es != start, ee != end), ep != period);
        let same = if d { false } else { true };
        if same {
            if e < res {
                res = e;
            }
        }
        e += 1;
    }
    res
}

/// All runs of `s`, each recorded exactly once.
///
/// For every scan position `i` the prefix function of the suffix `s[i..]` is
/// built; a window length `L` then gives the minimal period
/// `p = L - pi[L - 1]` of `s[i..i + L)`.  That window is extended as far as
/// possible to the right (while `s[r + 1] == s[r + 1 - p]`) and to the left
/// (while `l > 0 && s[l - 1] == s[l - 1 + p]`, never past `i`).  When the
/// extended interval is long enough (`>= 2p`) and the minimal period of the
/// extended interval — recomputed from scratch — is still `p`, the interval is
/// a run and is appended unless it is already present.
pub fn runs(s: &[usize]) -> Vec<Run> {
    let n = s.len();
    let mut out: Vec<Run> = Vec::new();
    let mut i: usize = 0;
    while i < n {
        let m = n - i;
        let pi = prefix_pi(s, i, m);
        let mut l: usize = 1;
        while l <= m {
            let back = pi[l - 1];
            let p = l - back;
            // Extend the `p`-periodic window `[i, i + l)` to the right.
            // `r` is the inclusive last index of the window.
            let mut r = i + l - 1;
            let mut go = true;
            while go {
                let nr = r + 1;
                if nr < n {
                    let a = s[nr];
                    let b = s[nr - p];
                    if a == b {
                        r = nr;
                    } else {
                        go = false;
                    }
                } else {
                    go = false;
                }
            }
            // Extend it to the left, never below the scan point `i`.
            let mut l2 = i;
            let mut go2 = true;
            while go2 {
                if l2 > 0 {
                    let lm = l2 - 1;
                    let lp = lm + p;
                    let a = s[lm];
                    let b = s[lp];
                    if a == b {
                        l2 = lm;
                    } else {
                        go2 = false;
                    }
                } else {
                    go2 = false;
                }
            }
            let re = r + 1;
            let wlen = re - l2;
            let twop = 2 * p;
            if wlen >= twop {
                if l2 == i {
                    // Recheck the minimal period of the extended window.
                    let pi2 = prefix_pi(s, l2, wlen);
                    let back2 = pi2[wlen - 1];
                    let p2 = wlen - back2;
                    if p2 == p {
                        let k = dup_index(&out, l2, re, p);
                        let m2 = out.len();
                        if k == m2 {
                            out.push(Run { start: l2, end: re, period: p });
                        }
                    }
                }
            }
            l += 1;
        }
        i += 1;
    }
    out
}
