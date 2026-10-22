use string_lz77::LzFactor;

/// Longest common prefix of `s[q..]` and `s[p..]`, capped by the
/// non-overlapping constraint (`q + l <= p`) and the end of the string.
fn match_len_at(s: &[usize], q: usize, p: usize) -> usize {
    let n = s.len();
    let mut l = 0;
    let mut more = true;
    while more {
        let a = q + l;
        let b = p + l;
        if a < p && b < n && s[a] == s[b] {
            l += 1;
        } else {
            more = false;
        }
    }
    l
}

/// The length of the longest allowed match for `s[p..]`, i.e. the maximum of
/// `match_len_at s q p` over all earlier positions `q < p`.
fn max_match_len(s: &[usize], p: usize) -> usize {
    let mut best = 0;
    let mut q = 0;
    while q < p {
        let m = match_len_at(s, q, p);
        if m > best {
            best = m;
        }
        q += 1;
    }
    best
}

/// Rebuild the string from the factor list and compare it with `s`.
fn replay_ok(s: &[usize], fs: &Vec<LzFactor>) -> bool {
    let n = s.len();
    let mut out: Vec<usize> = Vec::new();
    let mut pos = 0;
    let mut i = 0;
    let mut ok = true;
    while i < fs.len() {
        let off = fs[i].offset;
        let len = fs[i].length;
        if off == 0 {
            if len == 1 {
                if pos < n {
                    let c = s[pos];
                    out.push(c);
                    pos += 1;
                } else {
                    ok = false;
                }
            } else {
                ok = false;
            }
        } else {
            if off <= pos {
                let end = pos + len;
                if end <= n {
                    let mut j = 0;
                    while j < len {
                        let src = pos - off + j;
                        let c = s[src];
                        out.push(c);
                        j += 1;
                    }
                    pos += len;
                } else {
                    ok = false;
                }
            } else {
                ok = false;
            }
        }
        i += 1;
    }
    if ok {
        if pos != n {
            ok = false;
        } else {
            let mut k = 0;
            let mut same = true;
            while k < out.len() && same {
                if out[k] == s[k] {
                    k += 1;
                } else {
                    same = false;
                }
            }
            ok = same;
        }
    }
    ok
}

/// Structural invariants: literals are `(0, 1)` and occur only where no match
/// exists; copy factors stay inside the already emitted history and have the
/// maximal allowed match length; the lengths sum to `|s|`.
fn factors_sound(s: &[usize], fs: &Vec<LzFactor>) -> bool {
    let n = s.len();
    let mut ok = true;
    let mut p = 0;
    let mut i = 0;
    while i < fs.len() {
        let off = fs[i].offset;
        let len = fs[i].length;
        let m = max_match_len(s, p);
        if off == 0 {
            if len != 1 {
                ok = false;
            }
            if m != 0 {
                ok = false;
            }
        } else {
            if off > p {
                ok = false;
            }
            if m != len {
                ok = false;
            }
        }
        p += len;
        i += 1;
    }
    if p != n {
        ok = false;
    }
    ok
}

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 0);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[0].length, 1);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}

#[test]
fn test_aaaa() {
    let s: [usize; 4] = [97, 97, 97, 97]; // "aaaa" -> (0,1) (1,1) (2,2)
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 3);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[0].length, 1);
    assert_eq!(fs[1].offset, 1);
    assert_eq!(fs[1].length, 1);
    assert_eq!(fs[2].offset, 2);
    assert_eq!(fs[2].length, 2);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}

#[test]
fn test_abababab() {
    let s: [usize; 8] = [97, 98, 97, 98, 97, 98, 97, 98]; // "abababab"
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 4);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[0].length, 1);
    assert_eq!(fs[1].offset, 0);
    assert_eq!(fs[1].length, 1);
    assert_eq!(fs[2].offset, 2);
    assert_eq!(fs[2].length, 2);
    assert_eq!(fs[3].offset, 4);
    assert_eq!(fs[3].length, 4);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}

#[test]
fn test_abcabcabc() {
    let s: [usize; 9] = [97, 98, 99, 97, 98, 99, 97, 98, 99]; // "abcabcabc"
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 5);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 0);
    assert_eq!(fs[2].offset, 0);
    assert_eq!(fs[3].offset, 3);
    assert_eq!(fs[3].length, 3);
    assert_eq!(fs[4].offset, 6);
    assert_eq!(fs[4].length, 3);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}

#[test]
fn test_aaaaaab() {
    let s: [usize; 7] = [97, 97, 97, 97, 97, 97, 98]; // "aaaaaab"
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 5);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 1);
    assert_eq!(fs[2].offset, 2);
    assert_eq!(fs[2].length, 2);
    assert_eq!(fs[3].offset, 4);
    assert_eq!(fs[3].length, 2);
    assert_eq!(fs[4].offset, 0);
    assert_eq!(fs[4].length, 1);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 5);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 0);
    assert_eq!(fs[2].offset, 0);
    assert_eq!(fs[3].offset, 2);
    assert_eq!(fs[3].length, 2);
    assert_eq!(fs[4].offset, 4);
    assert_eq!(fs[4].length, 1);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}

#[test]
fn test_abacabadabacaba() {
    let s: [usize; 15] = [
        97, 98, 97, 99, 97, 98, 97, 100, 97, 98, 97, 99, 97, 98, 97,
    ]; // "abacabadabacaba"
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 7);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 0);
    assert_eq!(fs[2].offset, 2);
    assert_eq!(fs[2].length, 1);
    assert_eq!(fs[3].offset, 0);
    assert_eq!(fs[4].offset, 4);
    assert_eq!(fs[4].length, 3);
    assert_eq!(fs[5].offset, 0);
    assert_eq!(fs[6].offset, 8);
    assert_eq!(fs[6].length, 7);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}

#[test]
fn test_all_literals() {
    // A strictly increasing string matches nothing: every factor is a literal.
    let s: [usize; 5] = [97, 98, 99, 100, 101]; // "abcde"
    let fs = string_lz77::lz77(&s);
    assert_eq!(fs.len(), 5);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 0);
    assert_eq!(fs[2].offset, 0);
    assert_eq!(fs[3].offset, 0);
    assert_eq!(fs[4].offset, 0);
    assert_eq!(fs[4].length, 1);
    assert!(replay_ok(&s, &fs));
    assert!(factors_sound(&s, &fs));
}
