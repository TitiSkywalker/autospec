//! Concrete, hand-checkable tests for the Lyndon array.
//!
//! Expected values were obtained from the definition `L[i] = max { l :
//! s[i..i+l] is Lyndon }` and cross-checked by the brute-force reference in the
//! crate's own (untranslated) test module.  Only flat `usize` arithmetic and
//! `while` loops are used, so the file stays inside the translatable subset.

/// Number of positions where `lyndon_array s` differs from `expect`
/// (also flags a length mismatch).
fn t_check(s: &[usize], expect: &[usize]) -> usize {
    let l = string_lyndon_array::lyndon_array(s);
    let mut bad: usize = 0;
    if l.len() != expect.len() {
        bad += 1;
    } else {
        let mut i = 0;
        while i < l.len() {
            let a = l[i];
            let b = expect[i];
            if a != b {
                bad += 1;
            }
            i += 1;
        }
    }
    bad
}

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let e: [usize; 0] = [];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let e: [usize; 1] = [1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_all_equal() {
    let s: [usize; 4] = [97, 97, 97, 97]; // "aaaa": no repeat is Lyndon
    let e: [usize; 4] = [1, 1, 1, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_increasing() {
    let s: [usize; 3] = [97, 98, 99]; // "abc": whole string is Lyndon
    let e: [usize; 3] = [3, 2, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_decreasing() {
    let s: [usize; 3] = [99, 98, 97]; // "cba": only single chars
    let e: [usize; 3] = [1, 1, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_abab() {
    let s: [usize; 4] = [97, 98, 97, 98]; // "abab" -> [2,1,2,1]
    let e: [usize; 4] = [2, 1, 2, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let e: [usize; 6] = [1, 2, 1, 2, 1, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_mississippi() {
    // "mississippi"
    let s: [usize; 11] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105];
    let e: [usize; 11] = [1, 3, 1, 1, 3, 1, 1, 3, 1, 1, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_max_not_first_failure() {
    // "aab": length 2 ("aa") is not Lyndon but length 3 ("aab") is, so the
    // answer is the largest Lyndon length, not the first failing one.
    let s: [usize; 3] = [97, 97, 98];
    let e: [usize; 3] = [3, 2, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_abcabc() {
    let s: [usize; 6] = [97, 98, 99, 97, 98, 99]; // "abcabc"
    let e: [usize; 6] = [3, 2, 1, 3, 2, 1];
    assert_eq!(t_check(&s, &e), 0);
}

#[test]
fn test_long_lyndon_prefix() {
    // "aabaaab": the longest Lyndon prefix starting at index 3 is the whole
    // remaining suffix "aaab" (length 4).
    let s: [usize; 7] = [97, 97, 98, 97, 97, 97, 98];
    let e: [usize; 7] = [3, 2, 1, 4, 3, 2, 1];
    assert_eq!(t_check(&s, &e), 0);
}
