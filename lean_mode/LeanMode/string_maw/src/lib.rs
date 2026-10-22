//! Minimal Absent Words (Crochemore–Mignosi–Resta).
//!
//! Characters are `usize` byte values (0..=255).
//!
//! A word `w = a·U·b` (with `a`, `b` single characters) is a *minimal absent
//! word* (MAW) of `s` when `w` does not occur in `s`, while both `a·U` and
//! `U·b` do occur in `s`; the characters `a` and `b` range over the alphabet of
//! characters occurring in `s`.
//!
//! `maw` enumerates all minimal absent words of `s` over that alphabet,
//! flat-encoded as `(data, lens)`: the words are concatenated in `data` and
//! `lens[i]` is the length of the `i`-th word (so splitting `data` by `lens`
//! recovers the list of words).

/// Is `s[i]` the first occurrence of its value in `s`?  Equivalently: does the
/// value `s[i]` occur in the prefix `s[0..i)`?
fn distinct_at(s: &[usize], i: usize) -> bool {
    let a = s[i];
    let mut fresh = true;
    let mut k = 0;
    while k < i {
        if s[k] == a {
            fresh = false;
        }
        k += 1;
    }
    fresh
}

/// Does `p` occur as a contiguous factor of `s`?
fn occurs(s: &[usize], p: &[usize]) -> bool {
    let n = s.len();
    let m = p.len();
    let mut found = false;
    let mut i = 0;
    while i < n {
        let mut j = 0;
        let mut matched = true;
        let mut more = true;
        while more {
            if j < m {
                let idx = i + j;
                if idx < n {
                    if s[idx] == p[j] {
                        j += 1;
                    } else {
                        matched = false;
                        more = false;
                    }
                } else {
                    matched = false;
                    more = false;
                }
            } else {
                more = false;
            }
        }
        if matched {
            found = true;
        }
        i += 1;
    }
    found
}

/// `[x] ++ u`.
fn prepend(x: usize, u: &[usize]) -> Vec<usize> {
    let mut r: Vec<usize> = Vec::new();
    r.push(x);
    let mut i = 0;
    while i < u.len() {
        let c = u[i];
        r.push(c);
        i += 1;
    }
    r
}

/// `u ++ [x]`.
fn postpend(u: &[usize], x: usize) -> Vec<usize> {
    let mut r: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < u.len() {
        let c = u[i];
        r.push(c);
        i += 1;
    }
    r.push(x);
    r
}

/// The factor `s[start .. start + len]` as a fresh vector.
fn factor(s: &[usize], start: usize, len: usize) -> Vec<usize> {
    let mut r: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < len {
        let ix = start + i;
        let c = s[ix];
        r.push(c);
        i += 1;
    }
    r
}

/// Is `w` already present among the flat-encoded words `(data, lens)`?
fn is_dup(data: &[usize], lens: &[usize], w: &[usize]) -> bool {
    let mut dup = false;
    let mut off = 0;
    let mut idx = 0;
    while idx < lens.len() {
        let li = lens[idx];
        if li == w.len() {
            let mut j = 0;
            let mut pos = off;
            let mut same = true;
            let mut more = true;
            while more {
                if j < li {
                    let dv = data[pos];
                    if dv == w[j] {
                        j += 1;
                        pos += 1;
                    } else {
                        same = false;
                        more = false;
                    }
                } else {
                    more = false;
                }
            }
            if same {
                dup = true;
            }
        }
        off += li;
        idx += 1;
    }
    dup
}

/// All minimal absent words of `s` over the alphabet of characters occurring
/// in `s`, flat-encoded as `(data, lens)`.
///
/// For every `a` in the distinct characters of `s`, every `b` in the distinct
/// characters of `s`, and every factor `U = s[start .. start + len]` (with
/// `start + len ≤ n`), form `w = [a] ++ U ++ [b]`.  Emit `w` (once) iff `w`
/// does not occur in `s`, while `[a] ++ U` and `U ++ [b]` both do.
pub fn maw(s: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let n = s.len();
    let mut data: Vec<usize> = Vec::new();
    let mut lens: Vec<usize> = Vec::new();

    let mut ai = 0;
    while ai < n {
        if distinct_at(s, ai) {
            let a = s[ai];
            let mut bi = 0;
            while bi < n {
                if distinct_at(s, bi) {
                    let b = s[bi];
                    let mut start = 0;
                    while start <= n {
                        let mut len = 0;
                        let mut go = true;
                        while go {
                            let end = start + len;
                            if end <= n {
                                let u = factor(s, start, len);
                                let ub = postpend(&u, b);
                                let au = prepend(a, &u);
                                let w = prepend(a, &ub);
                                let absent = !occurs(s, &w);
                                let aup = occurs(s, &au);
                                let ubp = occurs(s, &ub);
                                if absent {
                                    if aup {
                                        if ubp {
                                            let dup = is_dup(&data, &lens, &w);
                                            if !dup {
                                                let mut t = 0;
                                                while t < w.len() {
                                                    let c = w[t];
                                                    data.push(c);
                                                    t += 1;
                                                }
                                                let wl = w.len();
                                                lens.push(wl);
                                            }
                                        }
                                    }
                                }
                                len += 1;
                            } else {
                                go = false;
                            }
                        }
                        start += 1;
                    }
                }
                bi += 1;
            }
        }
        ai += 1;
    }
    (data, lens)
}
