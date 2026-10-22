//! Linear-time suffix array construction by SA-IS (induced sorting).
//!
//! Characters are `usize` values in `1..=255`; the algorithm appends a reserved
//! sentinel `0` internally and drops it from the result.  No `as` casts are
//! used, so the only numeric model involved is `usize` (→ `Std.Usize`), and all
//! buffers are flat `Vec`s whose index arithmetic is hoisted into `let`s before
//! being used as an indexed write.
//!
//! SA-IS has three moving parts:
//!   1. classify every position as S-type / L-type and find the LMS positions;
//!   2. induced sort (place a set of seeds, then induce L-suffixes left→right
//!      from bucket heads and S-suffixes right→left from bucket ends);
//!   3. reduce: name the sorted LMS substrings into a smaller string, recursing
//!      when two of them coincide, otherwise recurse is skipped.

/// Classify every position of `t` as S-type (`1`) or L-type (`0`).
///
/// The last position (the sentinel) is S-type; a position `i < n - 1` is S-type
/// iff `t[i] < t[i + 1]`, or `t[i] == t[i + 1]` and `i + 1` is S-type.
fn make_types(t: &[usize]) -> Vec<usize> {
    let n = t.len();
    let mut is_s: Vec<usize> = vec![0usize; n];
    let last = n - 1;
    is_s[last] = 1;
    let mut i = n - 1;
    while i > 0 {
        i -= 1;
        let a = t[i];
        let b = t[i + 1];
        if a < b {
            is_s[i] = 1;
        } else if a > b {
            is_s[i] = 0;
        } else {
            let s = is_s[i + 1];
            is_s[i] = s;
        }
    }
    is_s
}

/// Do the two LMS substrings starting at `l` and `r` agree?
///
/// The substrings span `l..=end_l` and `r..=end_r` (the character of the next
/// LMS position is included).  Callers guarantee `l ≤ end_l` and `r ≤ end_r`.
fn span_eq(t: &[usize], l: usize, r: usize, end_l: usize, end_r: usize) -> bool {
    let dl = end_l - l;
    let dr = end_r - r;
    let mut eq = true;
    if dl != dr {
        eq = false;
    } else {
        let mut x = l;
        let mut y = r;
        let mut go = true;
        while go {
            let tx = t[x];
            let ty = t[y];
            if tx != ty {
                eq = false;
                go = false;
            } else if x < end_l {
                x = x + 1;
                y = y + 1;
            } else {
                go = false;
            }
        }
    }
    eq
}

/// Induced sort of the suffixes of `t` (classified by `is_s`), seeded by the
/// positions in `seed`.  `alpha` is an upper bound on the characters of `t`
/// (the number of buckets).
///
/// The buckets are laid out as [L-suffixes | S-suffixes]: the seed (the LMS
/// positions) is placed at the bucket ends, L-suffixes are induced into the
/// bucket heads scanning left→right, and S-suffixes are induced into the bucket
/// ends scanning right→left.
fn induced_sort(t: &[usize], is_s: &[usize], seed: &[usize], alpha: usize) -> Vec<usize> {
    let n = t.len();

    // Bucket sizes.
    let mut cnt: Vec<usize> = vec![0usize; alpha];
    let mut i = 0;
    while i < n {
        let c = t[i];
        let v = cnt[c];
        let v1 = v + 1;
        cnt[c] = v1;
        i += 1;
    }

    // `head[c]` is the first slot of bucket `c`; `end[c]` is one past its end.
    let mut head: Vec<usize> = vec![0usize; alpha];
    let mut end: Vec<usize> = vec![0usize; alpha];
    let mut acc = 0;
    let mut c0 = 0;
    while c0 < alpha {
        head[c0] = acc;
        let cc = cnt[c0];
        let e = acc + cc;
        end[c0] = e;
        acc = e;
        c0 += 1;
    }

    // `empty` marks a slot that has not been filled (valid positions are 0..n-1).
    let empty = n;
    let mut sa: Vec<usize> = vec![empty; n];

    // 1. Place the seed at the bucket ends, last seed first so that the first
    //    seed lands lowest within its bucket.
    let mut bend: Vec<usize> = vec![0usize; alpha];
    let mut c1 = 0;
    while c1 < alpha {
        let e = end[c1];
        bend[c1] = e;
        c1 += 1;
    }
    let sl = seed.len();
    let mut k = sl;
    while k > 0 {
        k -= 1;
        let p = seed[k];
        let cp = t[p];
        let e = bend[cp];
        let e1 = e - 1;
        sa[e1] = p;
        bend[cp] = e1;
    }

    // 2. Induce the L-type suffixes into the bucket heads, left to right.
    let mut bhead: Vec<usize> = vec![0usize; alpha];
    let mut c2 = 0;
    while c2 < alpha {
        let h = head[c2];
        bhead[c2] = h;
        c2 += 1;
    }
    let mut i2 = 0;
    while i2 < n {
        let v = sa[i2];
        if v < empty {
            if v >= 1 {
                let j = v - 1;
                let sj = is_s[j];
                if sj == 0 {
                    let cj = t[j];
                    let h = bhead[cj];
                    sa[h] = j;
                    let h1 = h + 1;
                    bhead[cj] = h1;
                }
            }
        }
        i2 += 1;
    }

    // 3. Induce the S-type suffixes into the bucket ends, right to left.
    let mut send: Vec<usize> = vec![0usize; alpha];
    let mut c3 = 0;
    while c3 < alpha {
        let e = end[c3];
        send[c3] = e;
        c3 += 1;
    }
    let mut i3 = n;
    while i3 > 0 {
        i3 -= 1;
        let v = sa[i3];
        if v < empty {
            if v >= 1 {
                let j = v - 1;
                let sj = is_s[j];
                if sj == 1 {
                    let cj = t[j];
                    let e = send[cj];
                    let e1 = e - 1;
                    sa[e1] = j;
                    send[cj] = e1;
                }
            }
        }
    }

    sa
}

/// Recursive core of SA-IS: the suffix array of `t`, whose last character is
/// the unique smallest sentinel.  `alpha` bounds the characters of `t`.
fn sais_rec(t: &[usize], alpha: usize) -> Vec<usize> {
    let n = t.len();
    if n == 1 {
        let mut one: Vec<usize> = vec![0usize; 1];
        return one;
    }

    let is_s = make_types(t);

    // LMS positions (S-type preceded by L-type) in text order.
    let mut is_lms: Vec<usize> = vec![0usize; n];
    let mut lms: Vec<usize> = Vec::new();
    let mut i = 1;
    while i < n {
        let cur = is_s[i];
        let prv = is_s[i - 1];
        if cur == 1 {
            if prv == 0 {
                is_lms[i] = 1;
                lms.push(i);
            }
        }
        i += 1;
    }
    let m = lms.len();

    // Stage 1: induced sort seeded by the LMS positions in text order; the
    // resulting order sorts the LMS substrings.
    let sa1 = induced_sort(t, &is_s, &lms, alpha);

    let mut sorted_lms: Vec<usize> = Vec::new();
    let mut j = 0;
    while j < n {
        let v = sa1[j];
        let lm = is_lms[v];
        if lm == 1 {
            sorted_lms.push(v);
        }
        j += 1;
    }

    // `lms_map[p]` is the text-order index of the LMS position `p`.
    let mut lms_map: Vec<usize> = vec![0usize; n];
    let mut q = 0;
    while q < m {
        let p = lms[q];
        lms_map[p] = q;
        q += 1;
    }

    // Name the sorted LMS substrings; equal substrings share a name.
    let mut rec_s: Vec<usize> = vec![0usize; m];
    let mut name = 0;
    let first = sorted_lms[0];
    let fidx = lms_map[first];
    rec_s[fidx] = 0;
    let mut a = 1;
    while a < m {
        let l = sorted_lms[a - 1];
        let r = sorted_lms[a];
        let li = lms_map[l];
        let li1 = li + 1;
        let end_l = if li1 < m { lms[li1] } else { n - 1 };
        let ri = lms_map[r];
        let ri1 = ri + 1;
        let end_r = if ri1 < m { lms[ri1] } else { n - 1 };
        let same = span_eq(t, l, r, end_l, end_r);
        if same {
            // reuse the current name
        } else {
            let name1 = name + 1;
            name = name1;
        }
        let ridx = lms_map[r];
        rec_s[ridx] = name;
        a += 1;
    }

    // If every name is distinct the sorted LMS order is already `sorted_lms`;
    // otherwise recurse on the reduced string `rec_s`.
    let mut final_lms: Vec<usize> = Vec::new();
    let all_distinct = name + 1;
    if all_distinct == m {
        final_lms = sorted_lms;
    } else {
        let rec_alpha = name + 1;
        let rec_sa = sais_rec(&rec_s, rec_alpha);
        let mut fl: Vec<usize> = vec![0usize; m];
        let mut b = 0;
        while b < m {
            let idx = rec_sa[b];
            let p = lms[idx];
            fl[b] = p;
            b += 1;
        }
        final_lms = fl;
    }

    // Stage 2: final induced sort from the sorted LMS positions.
    induced_sort(t, &is_s, &final_lms, alpha)
}

/// Suffix array of `s` by SA-IS (induced sorting): `sa[i]` is the start of the
/// `i`-th lexicographically smallest suffix of `s`.
///
/// Characters are `usize` values in `1..=255`; a sentinel `0` is appended
/// internally and dropped from the result.
pub fn sais(s: &[usize]) -> Vec<usize> {
    let n = s.len();

    // Append the sentinel `0`.
    let mut t: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        let x = s[i];
        t.push(x);
        i += 1;
    }
    t.push(0usize);

    let sa = sais_rec(&t, 256usize);

    // Drop the sentinel suffix (the unique smallest, at `sa[0]`).
    let m = sa.len();
    let mut out: Vec<usize> = Vec::new();
    let mut j = 1;
    while j < m {
        let v = sa[j];
        out.push(v);
        j += 1;
    }
    out
}
