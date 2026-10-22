//! Tests for the minimal-absent-words enumeration.
//!
//! The reference (`w_brute_maw`) enumerates *every* word over the alphabet of
//! `s` up to the length bound `|s| + 1` and applies the definition directly
//! (a word `w` is a MAW iff `w` is absent while its prefix `w[0..|w|-1]` and
//! its suffix `w[1..|w|]` both occur in `s`).  This is independent of the
//! factor-based enumeration in `src/lib.rs`.

/// Does `p` occur as a contiguous factor of `s`?  (Reference version.)
fn w_occurs(s: &[usize], p: &[usize]) -> bool {
    let n = s.len();
    let m = p.len();
    let mut found = false;
    let mut start = 0;
    while start <= n {
        let end = start + m;
        if end <= n {
            let mut ok = true;
            let mut j = 0;
            while j < m {
                let ix = start + j;
                let pj = p[j];
                if s[ix] != pj {
                    ok = false;
                }
                j += 1;
            }
            if ok {
                found = true;
            }
        }
        start += 1;
    }
    found
}

/// The distinct characters of `s`, in order of first appearance.
fn w_alpha(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut alpha: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        let c = s[i];
        let mut seen = false;
        let mut k = 0;
        while k < alpha.len() {
            let d = alpha[k];
            if d == c {
                seen = true;
            }
            k += 1;
        }
        if !seen {
            alpha.push(c);
        }
        i += 1;
    }
    alpha
}

/// Brute-force enumeration of all minimal absent words of `s` over the
/// alphabet of `s`, flat-encoded as `(data, lens)`.
///
/// Every MAW `w = a·U·b` satisfies `|a·U| ≤ |s|`, hence `|w| ≤ |s| + 1`; and
/// every character of `w` occurs in `s`.  So it suffices to try every word over
/// the alphabet of length `2 ..= |s| + 1` and keep those that are absent while
/// both their one-character-shorter prefix and suffix occur.
fn w_brute_maw(s: &[usize]) -> (Vec<usize>, Vec<usize>) {
    let n = s.len();
    let alpha = w_alpha(s);
    let m = alpha.len();
    let mut data: Vec<usize> = Vec::new();
    let mut lens: Vec<usize> = Vec::new();

    if m > 0 {
        let mut len = 2;
        while len <= n + 1 {
            let mut digits: Vec<usize> = vec![0usize; len];
            let mut done = false;
            while !done {
                // Build the word `w` and its prefix/suffix (drop last / drop first).
                let mut w: Vec<usize> = Vec::new();
                let mut prefix: Vec<usize> = Vec::new();
                let mut suffix: Vec<usize> = Vec::new();
                let mut t = 0;
                while t < len {
                    let di = digits[t];
                    let c = alpha[di];
                    w.push(c);
                    let t1 = t + 1;
                    if t1 < len {
                        prefix.push(c);
                    }
                    if t > 0 {
                        suffix.push(c);
                    }
                    t += 1;
                }
                let absent = !w_occurs(s, &w);
                let pre_here = w_occurs(s, &prefix);
                let suf_here = w_occurs(s, &suffix);
                if absent {
                    if pre_here {
                        if suf_here {
                            let mut q = 0;
                            while q < len {
                                let c = w[q];
                                data.push(c);
                                q += 1;
                            }
                            lens.push(len);
                        }
                    }
                }
                // Odometer increment of `digits` (base `m`).
                let mut pos = len;
                let mut carry = true;
                while carry && pos > 0 {
                    pos -= 1;
                    let d = digits[pos];
                    let d2 = d + 1;
                    if d2 < m {
                        digits[pos] = d2;
                        carry = false;
                    } else {
                        digits[pos] = 0;
                    }
                }
                if carry {
                    done = true;
                }
            }
            len += 1;
        }
    }
    (data, lens)
}

/// Is the `k`-th word of the flat encoding `(data, lens)` also present in the
/// flat encoding `(data2, lens2)`?
fn w_word_in(
    data: &[usize],
    lens: &[usize],
    k: usize,
    data2: &[usize],
    lens2: &[usize],
) -> bool {
    let mut off = 0;
    let mut i = 0;
    while i < k {
        let li = lens[i];
        off += li;
        i += 1;
    }
    let len = lens[k];

    let mut found = false;
    let mut off2 = 0;
    let mut j = 0;
    while j < lens2.len() {
        let len2 = lens2[j];
        if len2 == len {
            let mut t = 0;
            let mut same = true;
            let mut p = off;
            let mut q = off2;
            while t < len {
                let dv = data[p];
                let dw = data2[q];
                if dv == dw {
                    p += 1;
                    q += 1;
                    t += 1;
                } else {
                    same = false;
                    t = len;
                }
            }
            if same {
                found = true;
            }
        }
        off2 += len2;
        j += 1;
    }
    found
}

/// Do the two flat encodings represent the same *set* of words?
fn w_same_set(d1: &[usize], l1: &[usize], d2: &[usize], l2: &[usize]) -> bool {
    let mut ok = true;
    if l1.len() != l2.len() {
        ok = false;
    }
    let mut i = 0;
    while i < l1.len() {
        let f = w_word_in(d1, l1, i, d2, l2);
        if !f {
            ok = false;
        }
        i += 1;
    }
    let mut j = 0;
    while j < l2.len() {
        let f = w_word_in(d2, l2, j, d1, l1);
        if !f {
            ok = false;
        }
        j += 1;
    }
    ok
}

/// None of the emitted words occurs in `s`.
fn w_absent_all(s: &[usize], data: &[usize], lens: &[usize]) -> bool {
    let mut ok = true;
    let mut off = 0;
    let mut i = 0;
    while i < lens.len() {
        let l = lens[i];
        let mut w: Vec<usize> = Vec::new();
        let mut j = 0;
        let mut p = off;
        while j < l {
            let c = data[p];
            w.push(c);
            p += 1;
            j += 1;
        }
        let occ = w_occurs(s, &w);
        if occ {
            ok = false;
        }
        off += l;
        i += 1;
    }
    ok
}

/// Every word has length at least two, and the lengths sum to `data.len()`.
fn w_lens_ok(data: &[usize], lens: &[usize]) -> bool {
    let mut ok = true;
    let mut sum = 0;
    let mut i = 0;
    while i < lens.len() {
        let l = lens[i];
        if l < 2 {
            ok = false;
        }
        sum += l;
        i += 1;
    }
    if sum != data.len() {
        ok = false;
    }
    ok
}

/// Full check for one input: set equality with the brute-force reference, no
/// emitted word occurs in `s`, and the flat encoding is well-formed.
fn w_check(s: &[usize]) -> bool {
    let (data, lens) = string_maw::maw(s);
    let (bd, bl) = w_brute_maw(s);
    let mut ok = true;
    if !w_same_set(&data, &lens, &bd, &bl) {
        ok = false;
    }
    if !w_absent_all(s, &data, &lens) {
        ok = false;
    }
    if !w_lens_ok(&data, &lens) {
        ok = false;
    }
    ok
}

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let (data, lens) = string_maw::maw(&s);
    assert_eq!(data.len(), 0);
    assert_eq!(lens.len(), 0);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a" -> { "aa" }
    let r = w_check(&s);
    assert!(r);
    let (data, lens) = string_maw::maw(&s);
    assert_eq!(lens.len(), 1);
    assert_eq!(data.len(), 2);
}

#[test]
fn test_aa() {
    let s: [usize; 2] = [97, 97]; // "aa"
    let r = w_check(&s);
    assert!(r);
}

#[test]
fn test_ab() {
    let s: [usize; 2] = [97, 98]; // "ab" -> { "aa", "ba", "bb" }
    let r = w_check(&s);
    assert!(r);
    let (data, lens) = string_maw::maw(&s);
    assert_eq!(lens.len(), 3);
    assert_eq!(data.len(), 6);
}

#[test]
fn test_aab() {
    let s: [usize; 3] = [97, 97, 98]; // "aab"
    let r = w_check(&s);
    assert!(r);
}

#[test]
fn test_aba() {
    let s: [usize; 3] = [97, 98, 97]; // "aba"
    let r = w_check(&s);
    assert!(r);
}

#[test]
fn test_abba() {
    let s: [usize; 4] = [97, 98, 98, 97]; // "abba"
    let r = w_check(&s);
    assert!(r);
}

#[test]
fn test_abcab() {
    let s: [usize; 5] = [97, 98, 99, 97, 98]; // "abcab"
    let r = w_check(&s);
    assert!(r);
}

#[test]
fn test_aabaa() {
    let s: [usize; 5] = [97, 97, 98, 97, 97]; // "aabaa"
    let r = w_check(&s);
    assert!(r);
}

#[test]
fn test_abab() {
    let s: [usize; 4] = [97, 98, 97, 98]; // "abab"
    let r = w_check(&s);
    assert!(r);
}

#[test]
fn test_bytes() {
    let s: [usize; 4] = [0, 255, 0, 255]; // non-ASCII-ish bytes
    let r = w_check(&s);
    assert!(r);
}
