//! Tests for the palindromic tree (eertree).

/// Whether `s[start .. start + len]` reads the same forwards and backwards.
/// (`len` is always `>= 1` here.)
fn is_pal_window(s: &[usize], start: usize, len: usize) -> bool {
    let mut lo = start;
    let mut hi = start + len - 1;
    let mut isp = true;
    let mut done = false;
    while !done {
        if lo >= hi {
            done = true;
        } else {
            let a = s[lo];
            let b = s[hi];
            if a == b {
                lo += 1;
                hi -= 1;
            } else {
                isp = false;
                done = true;
            }
        }
    }
    isp
}

/// Whether the windows `s[a .. a + len]` and `s[b .. b + len]` are equal.
fn window_eq(s: &[usize], a: usize, b: usize, len: usize) -> bool {
    let mut k = 0;
    let mut eq = true;
    while k < len {
        let x = s[a + k];
        let y = s[b + k];
        if x != y {
            eq = false;
        }
        k += 1;
    }
    eq
}

/// Brute-force number of distinct nonempty palindromic substrings of `s`.
fn brute_count_pals(s: &[usize]) -> usize {
    let n = s.len();
    // palindromes found so far, stored as parallel `(start, length)` lists
    let mut pstarts: Vec<usize> = Vec::new();
    let mut plens: Vec<usize> = Vec::new();
    let mut count = 0;
    let mut l = 1;
    while l <= n {
        let mut i = 0;
        while i + l <= n {
            let ispal = is_pal_window(s, i, l);
            if ispal {
                // how many already-recorded palindromes equal this window?
                let mut matches: usize = 0;
                let mut k = 0;
                while k < pstarts.len() {
                    let lk = plens[k];
                    if lk == l {
                        let sk = pstarts[k];
                        let same = window_eq(s, sk, i, l);
                        if same {
                            matches += 1;
                        }
                    }
                    k += 1;
                }
                if matches == 0 {
                    pstarts.push(i);
                    plens.push(l);
                    count += 1;
                }
            }
            i += 1;
        }
        l += 1;
    }
    count
}

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 2);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 3);
    assert_eq!(t.len[2], 2); // shifted length of "a"
    assert_eq!(t.link[2], 1); // proper palindromic suffix "": the even root
}

#[test]
fn test_aa() {
    let s: [usize; 2] = [97, 97]; // "aa": a, aa
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 4);
    assert_eq!(t.len[3], 3); // shifted length of "aa"
    assert_eq!(t.link[3], 2); // proper palindromic suffix "a"
}

#[test]
fn test_ab() {
    let s: [usize; 2] = [97, 98]; // "ab": a, b
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 4);
}

#[test]
fn test_aba() {
    let s: [usize; 3] = [97, 98, 97]; // "aba": a, b, aba
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 5);
    assert_eq!(t.len[4], 4); // shifted length of "aba"
    assert_eq!(t.link[4], 2); // proper palindromic suffix "a"
}

#[test]
fn test_abba() {
    let s: [usize; 4] = [97, 98, 98, 97]; // "abba": a, b, bb, abba
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 6);
    assert_eq!(t.link[5], 2); // proper palindromic suffix of "abba" is "a"
}

#[test]
fn test_aaa() {
    let s: [usize; 3] = [97, 97, 97]; // "aaa": a, aa, aaa
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 5);
    assert_eq!(t.link[2], 1); // "a"  -> ""
    assert_eq!(t.link[3], 2); // "aa" -> "a"
    assert_eq!(t.link[4], 3); // "aaa" -> "aa"
}

#[test]
fn test_abacaba() {
    let s: [usize; 7] = [97, 98, 97, 99, 97, 98, 97]; // "abacaba"
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 9);
}

#[test]
fn test_abccba() {
    let s: [usize; 6] = [97, 98, 99, 99, 98, 97]; // "abccba": a,b,c,cc,bccb,abccba
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 8);
}

#[test]
fn test_aabbaa() {
    let s: [usize; 6] = [97, 97, 98, 98, 97, 97]; // "aabbaa"
    let t = string_palindromic_tree::eertree_build(&s);
    assert_eq!(t.sz, 8);
}

#[test]
fn test_links_point_below() {
    let s: [usize; 7] = [97, 98, 97, 99, 97, 98, 97]; // "abacaba"
    let t = string_palindromic_tree::eertree_build(&s);
    let sz = t.sz;
    let mut bad: usize = 0;
    let mut v = 2;
    while v < sz {
        let lk = t.link[v];
        if lk >= v {
            bad += 1;
        }
        v += 1;
    }
    assert_eq!(bad, 0);
}

#[test]
fn test_brute_aaa() {
    let s: [usize; 3] = [97, 97, 97]; // "aaa"
    let t = string_palindromic_tree::eertree_build(&s);
    let bc = brute_count_pals(&s);
    assert_eq!(bc, 3);
    assert_eq!(t.sz, bc + 2);
}

#[test]
fn test_brute_matches_count() {
    let s1: [usize; 6] = [97, 97, 98, 98, 97, 97]; // "aabbaa"
    let t1 = string_palindromic_tree::eertree_build(&s1);
    let b1 = brute_count_pals(&s1);
    assert_eq!(t1.sz, b1 + 2);

    let s2: [usize; 7] = [97, 98, 97, 99, 97, 98, 97]; // "abacaba"
    let t2 = string_palindromic_tree::eertree_build(&s2);
    let b2 = brute_count_pals(&s2);
    assert_eq!(t2.sz, b2 + 2);

    let s3: [usize; 6] = [97, 98, 99, 99, 98, 97]; // "abccba"
    let t3 = string_palindromic_tree::eertree_build(&s3);
    let b3 = brute_count_pals(&s3);
    assert_eq!(t3.sz, b3 + 2);
}

#[test]
fn test_brute_small() {
    let s1: [usize; 0] = [];
    let t1 = string_palindromic_tree::eertree_build(&s1);
    let b1 = brute_count_pals(&s1);
    assert_eq!(b1, 0);
    assert_eq!(t1.sz, b1 + 2);

    let s2: [usize; 3] = [97, 98, 97]; // "aba"
    let t2 = string_palindromic_tree::eertree_build(&s2);
    let b2 = brute_count_pals(&s2);
    assert_eq!(b2, 3);
    assert_eq!(t2.sz, b2 + 2);

    let s3: [usize; 4] = [97, 98, 98, 97]; // "abba"
    let t3 = string_palindromic_tree::eertree_build(&s3);
    let b3 = brute_count_pals(&s3);
    assert_eq!(b3, 4);
    assert_eq!(t3.sz, b3 + 2);
}
