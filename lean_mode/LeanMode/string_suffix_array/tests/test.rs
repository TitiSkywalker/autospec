//! Tests for the suffix array (prefix doubling) and the Kasai LCP array.
//!
//! Every case is cross-checked against an `O(n^2)` brute-force reference
//! (`ref_sa` / `ref_lcp`) and, for a few inputs, against hand-computed values.

use string_suffix_array::{build_sa, kasai_lcp};

// ----------------------- brute-force reference -----------------------

/// Lexicographic comparison of the suffixes `s[a..]` and `s[b..]`: `-1` if the
/// first is smaller, `1` if larger (suffixes of one string are never equal).
fn ref_suffix_cmp(s: &[usize], a: usize, b: usize) -> i32 {
    let n = s.len();
    let mut i = a;
    let mut j = b;
    let mut res = 0;
    let mut done = false;
    while !done {
        let in_i = i < n;
        let in_j = j < n;
        if in_i && in_j {
            let x = s[i];
            let y = s[j];
            if x < y {
                res = -1;
                done = true;
            } else if x > y {
                res = 1;
                done = true;
            } else {
                i += 1;
                j += 1;
            }
        } else if in_i {
            // `s[b..]` ran out first, so it is a proper prefix and is smaller.
            res = 1;
            done = true;
        } else if in_j {
            res = -1;
            done = true;
        } else {
            res = 0;
            done = true;
        }
    }
    res
}

/// Number of suffixes that are strictly smaller than the suffix at `i`.
fn ref_rank(s: &[usize], i: usize) -> usize {
    let n = s.len();
    let mut c = 0;
    let mut j = 0;
    while j < n {
        if j != i {
            let r = ref_suffix_cmp(s, j, i);
            if r < 0 {
                let c1 = c + 1;
                c = c1;
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

/// Longest common prefix of the suffixes starting at `a` and at `b`.
fn ref_lcp_of(s: &[usize], a: usize, b: usize) -> usize {
    let n = s.len();
    let rest_a = n - a;
    let rest_b = n - b;
    let lim = if rest_a < rest_b { rest_a } else { rest_b };
    let mut h = 0;
    let mut go = true;
    while go {
        if h < lim {
            let x = s[a + h];
            let y = s[b + h];
            if x == y {
                let h1 = h + 1;
                h = h1;
            } else {
                go = false;
            }
        } else {
            go = false;
        }
    }
    h
}

/// The LCP array of `sa`, computed by brute-force comparisons.
fn ref_lcp(s: &[usize], sa: &[usize]) -> Vec<usize> {
    let n = sa.len();
    let mut lcp: Vec<usize> = vec![0usize; n];
    let mut i = 0;
    while i + 1 < n {
        let a = sa[i];
        let b = sa[i + 1];
        let h = ref_lcp_of(s, a, b);
        lcp[i] = h;
        i += 1;
    }
    lcp
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

/// Check `build_sa` and `kasai_lcp` against the brute-force reference.
fn check_case(s: &[usize]) {
    let n = s.len();
    let r = build_sa(s);
    let sa = r.sa;
    let lcp = r.lcp;

    assert_eq!(sa.len(), n);
    assert_eq!(lcp.len(), n);

    let bsa = ref_sa(s);
    assert!(same_vec(&sa, &bsa));

    let blcp = ref_lcp(s, &bsa);
    assert!(same_vec(&lcp, &blcp));

    // `kasai_lcp` on its own agrees with the LCP array of `build_sa`.
    let l2 = kasai_lcp(s, &sa);
    assert!(same_vec(&l2, &blcp));
}

// ------------------------------ tests ------------------------------

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let r = build_sa(&s);
    let sa = r.sa;
    let lcp = r.lcp;
    assert_eq!(sa.len(), 0);
    assert_eq!(lcp.len(), 0);

    let l2 = kasai_lcp(&s, &sa);
    assert_eq!(l2.len(), 0);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let r = build_sa(&s);
    let sa = r.sa;
    let lcp = r.lcp;
    assert_eq!(sa.len(), 1);
    assert_eq!(sa[0], 0);
    assert_eq!(lcp.len(), 1);
    assert_eq!(lcp[0], 0);
}

#[test]
fn test_all_same() {
    let s: [usize; 4] = [97, 97, 97, 97]; // "aaaa"
    let r = build_sa(&s);
    let sa = r.sa;
    let lcp = r.lcp;
    assert_eq!(sa.len(), 4);
    assert_eq!(sa[0], 3);
    assert_eq!(sa[1], 2);
    assert_eq!(sa[2], 1);
    assert_eq!(sa[3], 0);
    assert_eq!(lcp[0], 1);
    assert_eq!(lcp[1], 2);
    assert_eq!(lcp[2], 3);
    assert_eq!(lcp[3], 0);
    check_case(&s);
}

#[test]
fn test_ababa() {
    let s: [usize; 5] = [97, 98, 97, 98, 97]; // "ababa"
    let r = build_sa(&s);
    let sa = r.sa;
    let lcp = r.lcp;
    assert_eq!(sa.len(), 5);
    assert_eq!(sa[0], 4); // "a"
    assert_eq!(sa[1], 2); // "aba"
    assert_eq!(sa[2], 0); // "ababa"
    assert_eq!(sa[3], 3); // "ba"
    assert_eq!(sa[4], 1); // "baba"
    assert_eq!(lcp[0], 1);
    assert_eq!(lcp[1], 3);
    assert_eq!(lcp[2], 0);
    assert_eq!(lcp[3], 2);
    assert_eq!(lcp[4], 0);
    check_case(&s);
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let r = build_sa(&s);
    let sa = r.sa;
    let lcp = r.lcp;
    assert_eq!(sa.len(), 6);
    // "a" | "ana" | "anana" | "banana" | "na" | "nana"
    assert_eq!(sa[0], 5);
    assert_eq!(sa[1], 3);
    assert_eq!(sa[2], 1);
    assert_eq!(sa[3], 0);
    assert_eq!(sa[4], 4);
    assert_eq!(sa[5], 2);
    // lcp("a","ana") = 1, lcp("ana","anana") = 3, lcp("anana","banana") = 0,
    // lcp("banana","na") = 0, lcp("na","nana") = 2, lcp[5] = 0.
    assert_eq!(lcp[0], 1);
    assert_eq!(lcp[1], 3);
    assert_eq!(lcp[2], 0);
    assert_eq!(lcp[3], 0);
    assert_eq!(lcp[4], 2);
    assert_eq!(lcp[5], 0);
    check_case(&s);
}

#[test]
fn test_mississippi() {
    let s: [usize; 11] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105];
    let expect: [usize; 11] = [10, 7, 4, 1, 0, 9, 8, 6, 3, 5, 2];
    let r = build_sa(&s);
    let sa = r.sa;
    let lcp = r.lcp;
    assert_eq!(sa.len(), 11);
    assert!(same_vec(&sa, &expect));
    let expect_lcp: [usize; 11] = [1, 1, 4, 0, 0, 1, 0, 2, 1, 3, 0];
    assert!(same_vec(&lcp, &expect_lcp));
    check_case(&s);
}

#[test]
fn test_abracadabra() {
    let s: [usize; 11] = [97, 98, 114, 97, 99, 97, 100, 97, 98, 114, 97];
    let expect: [usize; 11] = [10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2];
    let r = build_sa(&s);
    let sa = r.sa;
    assert_eq!(sa.len(), 11);
    assert!(same_vec(&sa, &expect));
    check_case(&s);
}

#[test]
fn test_aabaaaab() {
    let s: [usize; 8] = [97, 97, 98, 97, 97, 97, 97, 98]; // "aabaaaab"
    let expect: [usize; 8] = [3, 4, 5, 0, 6, 1, 7, 2];
    let r = build_sa(&s);
    let sa = r.sa;
    assert_eq!(sa.len(), 8);
    assert!(same_vec(&sa, &expect));
    check_case(&s);
}

#[test]
fn test_long_doubling() {
    // 16 characters: forces the doubling loop to run for k = 1, 2, 4, 8.
    let s: [usize; 16] = [
        97, 98, 97, 98, 97, 98, 97, 98, 97, 98, 97, 98, 97, 98, 97, 99,
    ];
    check_case(&s);
}

#[test]
fn test_kasai_direct() {
    // "banana": sa = [5, 3, 1, 0, 4, 2] (given, not computed).
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97];
    let sa: [usize; 6] = [5, 3, 1, 0, 4, 2];
    let l = kasai_lcp(&s, &sa);
    assert_eq!(l.len(), 6);
    assert_eq!(l[0], 1);
    assert_eq!(l[1], 3);
    assert_eq!(l[2], 0);
    assert_eq!(l[3], 0);
    assert_eq!(l[4], 2);
    assert_eq!(l[5], 0);
}
