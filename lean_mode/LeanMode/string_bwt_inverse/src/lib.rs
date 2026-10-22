//! Burrows-Wheeler transform (BWT) with its LF-mapping inverse.
//!
//! ## Alphabet convention
//!
//! Every input character is a `usize` byte value in `1..=255`.  The value `0`
//! is **reserved as a sentinel**: it is strictly smaller than every character
//! and it never occurs in the input.
//!
//! `bwt_encode` appends exactly one sentinel to the input before sorting the
//! cyclic rotations.  Because the sentinel occurs exactly once and is strict
//! minimum, all `n + 1` rotations are pairwise distinct, so the lexicographic
//! order of the rotations is total and `bwt_decode` needs no extra index.

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
/// index `start`, i.e. `t[(start + m - 1) % m]`: `t[m - 1]` when `start == 0`
/// and `t[start - 1]` otherwise.
fn rotation_last_char(t: &[usize], m: usize, start: usize) -> usize {
    let p = if start == 0 { m - 1 } else { start - 1 };
    t[p]
}

/// Lexicographic order of the cyclic rotations of `t` (of length `m`) starting
/// at `a` and at `b`: returns `true` when rotation `a` is strictly smaller.
///
/// Both rotations are scanned in lock-step and the first position where they
/// differ decides the comparison.  The sentinel guarantees such a position
/// always exists, so `d` never reaches `m`.
fn rotation_lt(t: &[usize], m: usize, a: usize, b: usize) -> bool {
    let mut d = 0;
    let mut ca = 0;
    let mut cb = 0;
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

/// Burrows-Wheeler transform of `s`.
///
/// Let `t = s ++ [0]` and `m = s.len() + 1`.  The `m` cyclic rotations of `t`
/// are sorted lexicographically (all distinct, see the module comment) and the
/// **last column** of that sorted matrix is returned: element `r` of the result
/// is the last character of the rotation occupying row `r`.  Hence the result
/// always has length `s.len() + 1` and contains exactly one sentinel `0`.
pub fn bwt_encode(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let m = n + 1;

    // t = s ++ [0].
    let mut t: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        let ch = s[i];
        t.push(ch);
        i += 1;
    }
    t.push(0usize);

    // order = [0, 1, ..., m - 1], then selection-sort it by `rotation_lt`.
    let mut order: Vec<usize> = Vec::new();
    let mut j = 0;
    while j < m {
        order.push(j);
        j += 1;
    }

    let mut a = 0;
    while a < m {
        let mut best = a;
        let mut b = a + 1;
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

    // Last column of the sorted rotation matrix.
    let mut out: Vec<usize> = Vec::new();
    let mut r = 0;
    while r < m {
        let start = order[r];
        let ch = rotation_last_char(&t, m, start);
        out.push(ch);
        r += 1;
    }
    out
}

/// Inverse Burrows-Wheeler transform, via the LF mapping.
///
/// With `m = l.len()` and `C[c] = #{i < m : l[i] < c}` (for `c` in `0..=255`)
/// the LF mapping is `LF(i) = C[l[i]] + #{j < i : l[j] = l[i]}`.  Row `0` of
/// the sorted rotation matrix is the rotation starting with the sentinel (the
/// smallest character), and `LF` maps a row to the row of the rotation that
/// starts one position later, so walking `LF` `m` times from row `0` yields
/// `[0] ++ s` backwards.  The leading sentinel is dropped.
pub fn bwt_decode(l: &[usize]) -> Vec<usize> {
    let m = l.len();
    let mut out: Vec<usize> = Vec::new();
    if m == 0 {
        return out;
    }

    // occ[c] = #{i : l[i] = c}  (indices 0..=255).
    let mut occ: Vec<usize> = vec![0usize; 256];
    let mut i = 0;
    while i < m {
        let ch = l[i];
        let o = occ[ch];
        let o1 = o + 1;
        occ[ch] = o1;
        i += 1;
    }

    // cf[c] = C[c] = #{i : l[i] < c};  cf[0] = 0 and cf[c+1] = cf[c] + occ[c].
    let mut cf: Vec<usize> = vec![0usize; 256];
    let mut c = 0;
    while c < 255 {
        let acc = cf[c];
        let o = occ[c];
        let s1 = acc + o;
        let c1 = c + 1;
        cf[c1] = s1;
        c += 1;
    }

    // lf[i] = C[l[i]] + #{j < i : l[j] = l[i]}.
    let mut count: Vec<usize> = vec![0usize; 256];
    let mut lf: Vec<usize> = vec![0usize; m];
    let mut k = 0;
    while k < m {
        let ch = l[k];
        let base = cf[ch];
        let rank = count[ch];
        let v = base + rank;
        lf[k] = v;
        let rank1 = rank + 1;
        count[ch] = rank1;
        k += 1;
    }

    // Walk LF from row 0, writing the recovered sentinel-prefixed string
    // backwards into `buf`.
    let mut buf: Vec<usize> = vec![0usize; m];
    let mut p = 0;
    let mut q = m;
    while q > 0 {
        q -= 1;
        let ch = l[p];
        let np = lf[p];
        buf[q] = ch;
        p = np;
    }

    // buf = [0] ++ s: drop the leading sentinel.
    let mut w = 1;
    while w < m {
        let ch = buf[w];
        out.push(ch);
        w += 1;
    }
    out
}
