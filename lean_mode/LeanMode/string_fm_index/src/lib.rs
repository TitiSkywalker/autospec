//! FM-index (Ferragina–Manzini) with backward search.
//!
//! ## Alphabet convention
//!
//! Every input character is a `usize` byte value in `1..=255`.  The value `0`
//! is **reserved as a sentinel**: it is strictly smaller than every character
//! and never occurs in the input.
//!
//! For an input `s` we build the Burrows–Wheeler transform of `t = s ++ [0]`
//! (the selection sort of the cyclic rotations of `t` driven by `rotation_lt`,
//! as in `string_bwt_inverse`), together with
//!
//! * `c[ch]` — the number of BWT characters strictly smaller than `ch`
//!   (the first-column starting position of `ch`), and
//! * `occ[i * 256 + ch]` — the number of occurrences of `ch` in `bwt[0 .. i]`.
//!
//! `fm_backward_search` walks the pattern from right to left and keeps the
//! half-open suffix-array interval `[l, r)` of the suffixes of `s` that start
//! with the part of the pattern processed so far.  The interval is
//! **half-open**, so `l == r` means "no occurrence" and the number of
//! occurrences is always `r - l`.
//!
//! Because `t` carries one extra suffix (the bare sentinel at position
//! `s.len()`), the empty pattern is handled specially: it is reported as
//! `[0, s.len())`, i.e. all `s.len()` suffixes of `s`, which is exactly the
//! number of occurrences of the empty pattern.

/// `x + 1`, routed through a helper so the result stays a tracked `usize`.
fn inc(x: usize) -> usize {
    let y = x + 1;
    y
}

/// Flat index of `occ[i * 256 + ch]`.
fn occ_idx(i: usize, ch: usize) -> usize {
    let t = i * 256;
    let r = t + ch;
    r
}

/// `t[(a + d) % m]`.
///
/// Exact whenever `a < m` and `d < m`: then `a + d < 2m`, so subtracting `m`
/// once is enough and plain `%` (whose translation is awkward) is not needed.
fn rot_char(t: &[usize], m: usize, a: usize, d: usize) -> usize {
    let x = a + d;
    if x < m {
        t[x]
    } else {
        let y = x - m;
        t[y]
    }
}

/// The last character of the cyclic rotation of `t` (of length `m`) starting at
/// index `start`, i.e. `t[(start + m - 1) % m]`.
fn rotation_last_char(t: &[usize], m: usize, start: usize) -> usize {
    let p = if start == 0 { m - 1 } else { start - 1 };
    t[p]
}

/// Lexicographic order of the cyclic rotations of `t` (of length `m`) starting
/// at `a` and at `b`: `true` when rotation `a` is strictly smaller.
///
/// The sentinel guarantees that a differing position always exists, so `d`
/// never reaches `m`.
fn rotation_lt(t: &[usize], m: usize, a: usize, b: usize) -> bool {
    let mut d: usize = 0;
    let mut ca: usize = 0;
    let mut cb: usize = 0;
    let mut done = false;
    while !done {
        if d < m {
            let x = rot_char(t, m, a, d);
            let y = rot_char(t, m, b, d);
            if x != y {
                ca = x;
                cb = y;
                done = true;
            } else {
                d += 1;
            }
        } else {
            done = true;
        }
    }
    if ca < cb { true } else { false }
}

/// The FM-index of a string `s` (characters `1..=255`, sentinel `0`).
pub struct FmIndex {
    /// Burrows–Wheeler transform of `s ++ [0]` (length `s.len() + 1`).
    pub bwt: Vec<usize>,
    /// `c[ch]` = number of BWT characters strictly smaller than `ch`
    /// (length 256).
    pub c: Vec<usize>,
    /// Flat occurrence table: `occ[i * 256 + ch]` = number of occurrences of
    /// `ch` in `bwt[0 .. i]`, for `i` in `0 ..= s.len() + 1` (length
    /// `(s.len() + 2) * 256`).
    pub occ: Vec<usize>,
    /// `s.len()`.
    pub n: usize,
}

/// Build the FM-index of `s` (characters `1..=255`; the sentinel `0` is
/// appended internally).
pub fn fm_build(s: &[usize]) -> FmIndex {
    let n = s.len();
    let m = n + 1;

    // t = s ++ [0].
    let mut t: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < n {
        let ch = s[i];
        t.push(ch);
        i += 1;
    }
    t.push(0usize);

    // order = [0, 1, ..., m - 1], selection-sorted by `rotation_lt`.
    let mut order: Vec<usize> = Vec::new();
    let mut j: usize = 0;
    while j < m {
        order.push(j);
        j += 1;
    }
    let mut a: usize = 0;
    while a < m {
        let mut best: usize = a;
        let mut b: usize = a + 1;
        while b < m {
            let x = order[b];
            let y = order[best];
            let lt = rotation_lt(&t, m, x, y);
            if lt {
                best = b;
            }
            b += 1;
        }
        if best != a {
            let va = order[a];
            let vb = order[best];
            order[a] = vb;
            order[best] = va;
        }
        a += 1;
    }

    // bwt = last column of the sorted rotation matrix.
    let mut bwt: Vec<usize> = Vec::new();
    let mut r: usize = 0;
    while r < m {
        let start = order[r];
        let ch = rotation_last_char(&t, m, start);
        bwt.push(ch);
        r += 1;
    }

    // cnt[ch] = number of occurrences of ch in bwt.
    let mut cnt: Vec<usize> = vec![0usize; 256];
    let mut k: usize = 0;
    while k < m {
        let ch = bwt[k];
        let cur = cnt[ch];
        let nv = cur + 1;
        cnt[ch] = nv;
        k += 1;
    }

    // c[ch] = #{d < ch : cnt[d]}; c[0] = 0 and c[ch + 1] = c[ch] + cnt[ch].
    let mut c: Vec<usize> = vec![0usize; 256];
    let mut ch: usize = 0;
    while ch < 255 {
        let acc = c[ch];
        let occs = cnt[ch];
        let nv = acc + occs;
        let ch1 = ch + 1;
        c[ch1] = nv;
        ch = ch1;
    }

    // occ[i * 256 + ch] = #{j < i : bwt[j] == ch}, for i in 0 ..= m.
    //
    // Row 0 is all zeros and row i + 1 is row i with the entry of `bwt[i]`
    // incremented by one, so the table is **appended row by row**: the big
    // table is never randomly read or written while it is being built (only
    // the 256-entry running counter `run` is).  Appending a whole row at a
    // time with `extend_from_slice` keeps the construction linear in the
    // table size.
    let mut run: Vec<usize> = vec![0usize; 256];
    let mut occ: Vec<usize> = Vec::new();
    occ.extend_from_slice(&run);
    let mut row: usize = 0;
    while row < m {
        let bc = bwt[row];
        let cur = run[bc];
        let nv = cur + 1;
        run[bc] = nv;
        occ.extend_from_slice(&run);
        row += 1;
    }

    FmIndex { bwt, c, occ, n }
}

/// Backward search of the pattern `p` (Ferragina–Manzini).
///
/// Returns the half-open suffix-array interval `[l, r)` of the suffixes of `s`
/// that start with `p`; `l == r` when `p` does not occur.  The empty pattern
/// yields `[0, s.len())` (see the module comment).
pub fn fm_backward_search(idx: &FmIndex, p: &[usize]) -> (usize, usize) {
    let m = p.len();
    let nn = idx.n;
    let mut l: usize = 0;
    let mut r: usize = inc(nn);
    let mut j: usize = m;
    while j > 0 {
        j -= 1;
        let ch = p[j];
        let base = idx.c[ch];
        let ol = idx.occ[occ_idx(l, ch)];
        let nl = base + ol;
        let orr = idx.occ[occ_idx(r, ch)];
        let nr = base + orr;
        l = nl;
        r = nr;
    }
    if m == 0 {
        r = nn;
    }
    (l, r)
}
