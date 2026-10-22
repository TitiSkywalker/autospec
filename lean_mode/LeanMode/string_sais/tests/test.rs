//! Tests for the SA-IS suffix array.
//!
//! Every case is cross-checked against an `O(n^2)` brute-force reference
//! (`ref_sa`), against hand-computed values for `banana`/`mississippi`/etc.,
//! and the result is asserted to be a permutation of `0..n-1`.

use string_sais::sais;

// ----------------------- brute-force reference -----------------------

/// Is the suffix starting at `a` strictly lexicographically smaller than the
/// one starting at `b`?  (Suffixes of one string are never equal.)
fn ref_less(s: &[usize], a: usize, b: usize) -> bool {
    let n = s.len();
    let mut i = a;
    let mut j = b;
    let mut res = false;
    let mut done = false;
    while !done {
        let in_i = i < n;
        let in_j = j < n;
        if in_i {
            if in_j {
                let x = s[i];
                let y = s[j];
                if x < y {
                    res = true;
                    done = true;
                } else if x > y {
                    res = false;
                    done = true;
                } else {
                    i = i + 1;
                    j = j + 1;
                }
            } else {
                // `s[b..]` ran out first, so it is a proper prefix: not less.
                res = false;
                done = true;
            }
        } else {
            if in_j {
                // `s[a..]` ran out first, so it is a proper prefix: less.
                res = true;
            }
            done = true;
        }
    }
    res
}

/// Number of suffixes strictly smaller than the one starting at `i`.
fn ref_rank(s: &[usize], i: usize) -> usize {
    let n = s.len();
    let mut c: usize = 0;
    let mut j = 0;
    while j < n {
        if j != i {
            if ref_less(s, j, i) {
                c = c + 1;
            }
        }
        j += 1;
    }
    c
}

/// The suffix array, computed in `O(n^2)` by ranking every suffix.
fn ref_sa(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut sa: Vec<usize> = vec![0usize; n];
    let mut i = 0;
    while i < n {
        let r = ref_rank(s, i);
        sa[r] = i;
        i += 1;
    }
    sa
}

// --------------------------- comparison ---------------------------

/// Element-by-element equality of a `Vec` and a slice.
fn same_vec(a: &Vec<usize>, b: &[usize]) -> bool {
    let n = a.len();
    let m = b.len();
    let mut ok = true;
    let mut i = 0;
    while i < n {
        if i < m {
            let x = a[i];
            let y = b[i];
            if x != y {
                ok = false;
            }
        } else {
            ok = false;
        }
        i += 1;
    }
    if n != m {
        ok = false;
    }
    ok
}

/// Is `sa` a permutation of `0..n-1`?
fn is_perm(sa: &Vec<usize>, n: usize) -> bool {
    let mut seen: Vec<usize> = vec![0usize; n];
    let m = sa.len();
    let mut ok = true;
    if m != n {
        ok = false;
    }
    let mut i = 0;
    while i < m {
        let v = sa[i];
        if v < n {
            let s = seen[v];
            if s == 0 {
                seen[v] = 1;
            } else {
                ok = false;
            }
        } else {
            ok = false;
        }
        i += 1;
    }
    ok
}

/// Check `sais` against the brute-force reference and assert it is a perm.
fn check_case(s: &[usize]) {
    let n = s.len();
    let sa = sais(s);
    assert_eq!(sa.len(), n);
    assert!(is_perm(&sa, n));
    let bsa = ref_sa(s);
    assert!(same_vec(&sa, &bsa));
}

/// A deterministic pseudo-random-looking string over `1..=modulus`.
fn make_seq(n: usize, modulus: usize) -> Vec<usize> {
    let mut v: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < n {
        let t1 = i * 7;
        let t2 = (i / 3) * 13;
        let t3 = t1 + t2;
        let t4 = t3 + 5;
        let c = t4 % modulus;
        let c1 = c + 1;
        v.push(c1);
        i += 1;
    }
    v
}

// ------------------------------ tests ------------------------------

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let sa = sais(&s);
    assert_eq!(sa.len(), 0);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let sa = sais(&s);
    assert_eq!(sa.len(), 1);
    assert_eq!(sa[0], 0);
}

#[test]
fn test_two_same() {
    let s: [usize; 2] = [97, 97]; // "aa"
    let sa = sais(&s);
    assert_eq!(sa.len(), 2);
    assert_eq!(sa[0], 1);
    assert_eq!(sa[1], 0);
    check_case(&s);
}

#[test]
fn test_all_same() {
    let s: [usize; 4] = [97, 97, 97, 97]; // "aaaa"
    let sa = sais(&s);
    assert_eq!(sa.len(), 4);
    assert_eq!(sa[0], 3);
    assert_eq!(sa[1], 2);
    assert_eq!(sa[2], 1);
    assert_eq!(sa[3], 0);
    check_case(&s);
}

#[test]
fn test_abab() {
    let s: [usize; 4] = [97, 98, 97, 98]; // "abab"
    let sa = sais(&s);
    assert_eq!(sa.len(), 4);
    assert_eq!(sa[0], 2);
    assert_eq!(sa[1], 0);
    assert_eq!(sa[2], 3);
    assert_eq!(sa[3], 1);
    check_case(&s);
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let sa = sais(&s);
    assert_eq!(sa.len(), 6);
    // "a" | "ana" | "anana" | "banana" | "na" | "nana"
    assert_eq!(sa[0], 5);
    assert_eq!(sa[1], 3);
    assert_eq!(sa[2], 1);
    assert_eq!(sa[3], 0);
    assert_eq!(sa[4], 4);
    assert_eq!(sa[5], 2);
    check_case(&s);
}

#[test]
fn test_mississippi() {
    let s: [usize; 11] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105];
    let expect: [usize; 11] = [10, 7, 4, 1, 0, 9, 8, 6, 3, 5, 2];
    let sa = sais(&s);
    assert_eq!(sa.len(), 11);
    assert!(same_vec(&sa, &expect));
    check_case(&s);
}

#[test]
fn test_abracadabra() {
    let s: [usize; 11] = [97, 98, 114, 97, 99, 97, 100, 97, 98, 114, 97];
    let expect: [usize; 11] = [10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2];
    let sa = sais(&s);
    assert_eq!(sa.len(), 11);
    assert!(same_vec(&sa, &expect));
    check_case(&s);
}

#[test]
fn test_aabaaaab() {
    let s: [usize; 8] = [97, 97, 98, 97, 97, 97, 97, 98]; // "aabaaaab"
    let expect: [usize; 8] = [3, 4, 5, 0, 6, 1, 7, 2];
    let sa = sais(&s);
    assert_eq!(sa.len(), 8);
    assert!(same_vec(&sa, &expect));
    check_case(&s);
}

#[test]
fn test_long_pseudo_random() {
    // Repeated alphabet of size 5 over 60 characters: forces recursion.
    let s = make_seq(60, 5);
    check_case(&s);
    // Larger alphabet over 100 characters.
    let s2 = make_seq(100, 200);
    check_case(&s2);
}

#[test]
fn test_long_alternating() {
    let mut s: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < 64 {
        let c = i % 2;
        let c1 = c + 1;
        s.push(c1);
        i += 1;
    }
    check_case(&s);
}

#[test]
fn test_long_periodic() {
    // "abcd" repeated: many equal LMS substrings at every scale.
    let mut s: Vec<usize> = Vec::new();
    let mut i: usize = 0;
    while i < 80 {
        let c = i % 4;
        let c1 = c + 1;
        s.push(c1);
        i += 1;
    }
    check_case(&s);
}

