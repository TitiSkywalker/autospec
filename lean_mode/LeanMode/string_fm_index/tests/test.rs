// Characters are `usize` byte values in `1..=255`; `0` is the reserved sentinel.
//
// The helpers below deliberately use names that do not occur in `src/lib.rs`
// (`inc`, `occ_idx`, `occ_len`, `rot_char`, `rotation_last_char`,
// `rotation_lt`, `fm_build`, `fm_backward_search`).

/// Brute-force number of occurrences of the pattern `p` in the text `s`:
/// `p` occurs at position `i` when `i + p.len() <= s.len()` and
/// `s[i .. i + p.len()] == p`.  The empty pattern occurs at all `s.len()`
/// positions, which is exactly the number of suffixes of `s` prefixed by `p`.
fn brute_occurs(s: &[usize], p: &[usize]) -> usize {
    let n = s.len();
    let m = p.len();
    let mut count: usize = 0;
    let mut i: usize = 0;
    while i < n {
        let mut good = true;
        let fits = i + m <= n;
        if fits {
            let mut k: usize = 0;
            while k < m {
                let pos = i + k;
                let a = s[pos];
                let b = p[k];
                if a != b {
                    good = false;
                }
                k += 1;
            }
        } else {
            good = false;
        }
        if good {
            let c1 = count + 1;
            count = c1;
        }
        i += 1;
    }
    count
}

/// Compare `fm_backward_search` against the brute-force count for one pattern,
/// reusing an already built index.
fn check_idx_pair(s: &[usize], idx: &string_fm_index::FmIndex, p: &[usize]) {
    let (l, r) = string_fm_index::fm_backward_search(idx, p);
    assert!(l <= r);
    let mut cnt: usize = 0;
    let mut j: usize = l;
    while j < r {
        let c1 = cnt + 1;
        cnt = c1;
        j += 1;
    }
    let want = brute_occurs(s, p);
    assert_eq!(cnt, want);
}

/// Check every substring of `s` (including the empty one) as a pattern.
fn check_idx_substrings(s: &[usize], idx: &string_fm_index::FmIndex) {
    let n = s.len();
    let mut i: usize = 0;
    while i <= n {
        let mut j: usize = i;
        while j <= n {
            let mut pat: Vec<usize> = Vec::new();
            let mut k: usize = i;
            while k < j {
                let ch = s[k];
                pat.push(ch);
                k += 1;
            }
            check_idx_pair(s, idx, &pat);
            j += 1;
        }
        i += 1;
    }
}

#[test]
fn test_empty_text() {
    let s: [usize; 0] = [];
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 0] = [];
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 1] = [97];
    check_idx_pair(&s, &idx, &p1);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 1] = [98]; // "b", absent
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 2] = [97, 97]; // longer than the text
    check_idx_pair(&s, &idx, &p1);

    // The single suffix "a" occupies the interval [1, 2).
    let pa: [usize; 1] = [97];
    let (l, r) = string_fm_index::fm_backward_search(&idx, &pa);
    assert_eq!(l, 1);
    assert_eq!(r, 2);
}

#[test]
fn test_two_same() {
    let s: [usize; 2] = [97, 97]; // "aa"
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 3] = [97, 97, 97]; // longer than the text
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 1] = [98];
    check_idx_pair(&s, &idx, &p1);
}

#[test]
fn test_three_same() {
    let s: [usize; 3] = [97, 97, 97]; // "aaa"
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 4] = [97, 97, 97, 97];
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 2] = [97, 98];
    check_idx_pair(&s, &idx, &p1);
}

#[test]
fn test_two_distinct() {
    let s: [usize; 2] = [97, 98]; // "ab"
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 2] = [98, 97]; // "ba", absent
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 3] = [97, 98, 97];
    check_idx_pair(&s, &idx, &p1);
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 1] = [122]; // "z", absent
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 7] = [98, 97, 110, 97, 110, 97, 97]; // longer than the text
    check_idx_pair(&s, &idx, &p1);

    // "a" occurs at ranks 1, 2, 3 -> [1, 4); "an" at ranks 2, 3 -> [2, 4).
    let pa: [usize; 1] = [97];
    let (la, ra) = string_fm_index::fm_backward_search(&idx, &pa);
    assert_eq!(la, 1);
    assert_eq!(ra, 4);
    let pan: [usize; 2] = [97, 110];
    let (lan, ran) = string_fm_index::fm_backward_search(&idx, &pan);
    assert_eq!(lan, 2);
    assert_eq!(ran, 4);
}

#[test]
fn test_mississippi() {
    let s: [usize; 11] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105];
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 2] = [120, 121]; // "xy", absent
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 12] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105, 105];
    check_idx_pair(&s, &idx, &p1);
    let p2: [usize; 4] = [105, 115, 115, 105]; // "issi" occurs twice
    check_idx_pair(&s, &idx, &p2);
}

#[test]
fn test_abracadabra() {
    let s: [usize; 11] = [97, 98, 114, 97, 99, 97, 100, 97, 98, 114, 97];
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 2] = [97, 97]; // "aa", absent
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 3] = [97, 98, 114]; // "abr" occurs twice
    check_idx_pair(&s, &idx, &p1);
    let p2: [usize; 12] = [97, 98, 114, 97, 99, 97, 100, 97, 98, 114, 97, 97];
    check_idx_pair(&s, &idx, &p2);
}

#[test]
fn test_abcabcabc() {
    let s: [usize; 9] = [97, 98, 99, 97, 98, 99, 97, 98, 99];
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 4] = [99, 97, 98, 99]; // "cabc", occurs twice
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 10] = [97, 98, 99, 97, 98, 99, 97, 98, 99, 97];
    check_idx_pair(&s, &idx, &p1);
    let p2: [usize; 3] = [99, 98, 97]; // "cba", absent
    check_idx_pair(&s, &idx, &p2);
}

#[test]
fn test_high_byte_values() {
    let s: [usize; 5] = [255, 1, 200, 255, 1];
    let idx = string_fm_index::fm_build(&s);
    check_idx_substrings(&s, &idx);
    let p0: [usize; 2] = [255, 200]; // absent
    check_idx_pair(&s, &idx, &p0);
    let p1: [usize; 3] = [255, 1, 200];
    check_idx_pair(&s, &idx, &p1);
}
