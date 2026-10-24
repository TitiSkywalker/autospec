//! Concrete, small tests for the LZ-End factorization.  Translated to
//! `Test_test.lean`; the heavy brute-force checks live in the untranslated
//! `#[cfg(test)] mod tests` of `src/lib.rs`.

use string_lz_end::LzEndFactor;

/// Rebuild the string from the factor list and check it equals `s`; also check
/// every copy's source ends exactly at a factor boundary.  Returns `0` on
/// success, a non-zero error code otherwise.
fn t_replay_ok(s: &[usize], fs: &Vec<LzEndFactor>) -> usize {
    let n = s.len();
    let mut out: Vec<usize> = Vec::new();
    let mut bs: Vec<usize> = Vec::new();
    bs.push(0);
    let mut pos = 0;
    let mut i = 0;
    let mut bad: usize = 0;
    while i < fs.len() {
        let off = fs[i].offset;
        let len = fs[i].length;
        if len < 1 {
            bad = 1;
        }
        if off == 0 {
            if len != 1 {
                bad = 2;
            }
            if pos < n {
                let c = s[pos];
                out.push(c);
                pos += 1;
            } else {
                bad = 3;
            }
        } else {
            if off > pos {
                bad = 4;
            }
            let bend = pos - off + len;
            let mut found = false;
            let mut bi = 0;
            while bi < bs.len() {
                if bs[bi] == bend {
                    found = true;
                }
                bi += 1;
            }
            if !found {
                bad = 5;
            }
            if pos + len <= n {
                let mut j = 0;
                while j < len {
                    let c = s[pos - off + j];
                    out.push(c);
                    j += 1;
                }
                pos += len;
            } else {
                bad = 6;
            }
        }
        bs.push(pos);
        i += 1;
    }
    if pos != n {
        bad = 7;
    }
    if bad == 0 {
        let mut k = 0;
        while k < out.len() {
            if out[k] != s[k] {
                bad = 8;
            }
            k += 1;
        }
    }
    bad
}

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let fs = string_lz_end::lz_end(&s);
    assert_eq!(fs.len(), 0);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let fs = string_lz_end::lz_end(&s);
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[0].length, 1);
    assert_eq!(t_replay_ok(&s, &fs), 0);
}

#[test]
fn test_aa_aaa() {
    let s1: [usize; 2] = [97, 97]; // "aa" -> (0,1) (1,1)
    let f1 = string_lz_end::lz_end(&s1);
    assert_eq!(f1.len(), 2);
    assert_eq!(f1[0].offset, 0);
    assert_eq!(f1[0].length, 1);
    assert_eq!(f1[1].offset, 1);
    assert_eq!(f1[1].length, 1);
    assert_eq!(t_replay_ok(&s1, &f1), 0);

    let s2: [usize; 3] = [97, 97, 97]; // "aaa" -> (0,1) (1,1) (2,1)
    let f2 = string_lz_end::lz_end(&s2);
    assert_eq!(f2.len(), 3);
    assert_eq!(f2[0].offset, 0);
    assert_eq!(f2[1].offset, 1);
    assert_eq!(f2[2].offset, 2);
    assert_eq!(f2[2].length, 1);
    assert_eq!(t_replay_ok(&s2, &f2), 0);
}

#[test]
fn test_aaaa() {
    let s: [usize; 4] = [97, 97, 97, 97]; // "aaaa" -> (0,1) (1,1) (2,2)
    let fs = string_lz_end::lz_end(&s);
    assert_eq!(fs.len(), 3);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[0].length, 1);
    assert_eq!(fs[1].offset, 1);
    assert_eq!(fs[1].length, 1);
    assert_eq!(fs[2].offset, 2);
    assert_eq!(fs[2].length, 2);
    assert_eq!(t_replay_ok(&s, &fs), 0);
}

#[test]
fn test_abcabc() {
    let s: [usize; 6] = [97, 98, 99, 97, 98, 99]; // "abcabc"
    let fs = string_lz_end::lz_end(&s);
    assert_eq!(fs.len(), 4);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 0);
    assert_eq!(fs[2].offset, 0);
    assert_eq!(fs[3].offset, 3);
    assert_eq!(fs[3].length, 3);
    assert_eq!(t_replay_ok(&s, &fs), 0);
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let fs = string_lz_end::lz_end(&s);
    assert_eq!(fs.len(), 5);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 0);
    assert_eq!(fs[2].offset, 0);
    assert_eq!(fs[3].offset, 2);
    assert_eq!(fs[3].length, 2);
    assert_eq!(fs[4].offset, 4);
    assert_eq!(fs[4].length, 1);
    assert_eq!(t_replay_ok(&s, &fs), 0);
}

#[test]
fn test_nonoptimal_instance() {
    // Greedy LZ-End is not optimal: on [0,0,1,0,0,0,1,0] it emits 6 factors
    // (encoded here over 97/98), while a valid 5-factor parsing exists.
    let s: [usize; 8] = [97, 97, 98, 97, 97, 97, 98, 97];
    let fs = string_lz_end::lz_end(&s);
    assert_eq!(fs.len(), 6);
    assert_eq!(fs[0].offset, 0);
    assert_eq!(fs[1].offset, 1);
    assert_eq!(fs[1].length, 1);
    assert_eq!(fs[2].offset, 0);
    assert_eq!(fs[3].offset, 3);
    assert_eq!(fs[3].length, 2);
    assert_eq!(fs[4].offset, 4);
    assert_eq!(fs[4].length, 2);
    assert_eq!(fs[5].offset, 7);
    assert_eq!(fs[5].length, 1);
    assert_eq!(t_replay_ok(&s, &fs), 0);
}
