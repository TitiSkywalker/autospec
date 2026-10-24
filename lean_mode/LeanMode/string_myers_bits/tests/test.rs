//! Small, hand-checkable cases for the bit-parallel Shift-And matcher.
//!
//! Every function here is translated and `#eval`'d by CI, so it stays tiny and
//! uses only the translatable Rust subset.  The exhaustive/randomized
//! differential tests against a brute-force matcher live in the crate's own
//! (un-translated) `#[cfg(test)] mod tests`.

/// Number of disagreements between `r` and `expected` (length included).
fn t_check(r: &[usize], expected: &[usize]) -> usize {
    let mut bad: usize = 0;
    if r.len() != expected.len() {
        bad += 1;
    }
    let n = r.len();
    let mut i = 0;
    while i < n && i < expected.len() {
        if r[i] != expected[i] {
            bad += 1;
        }
        i += 1;
    }
    bad
}

#[test]
fn test_at_start() {
    let t: [usize; 5] = [1, 2, 3, 4, 5];
    let p: [usize; 3] = [1, 2, 3];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 1] = [0];
    assert_eq!(t_check(&r, &e), 0);
}

#[test]
fn test_at_end() {
    let t: [usize; 5] = [1, 2, 3, 4, 5];
    let p: [usize; 3] = [3, 4, 5];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 1] = [2];
    assert_eq!(t_check(&r, &e), 0);
}

#[test]
fn test_overlapping_aaa_aa() {
    // "aaa" with pattern "aa": occurrences at 0 and 1.
    let t: [usize; 3] = [97, 97, 97];
    let p: [usize; 2] = [97, 97];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 2] = [0, 1];
    assert_eq!(t_check(&r, &e), 0);
}

#[test]
fn test_no_occurrence() {
    let t: [usize; 3] = [1, 2, 3];
    let p: [usize; 2] = [9, 9];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 0] = [];
    assert_eq!(t_check(&r, &e), 0);
}

#[test]
fn test_pattern_longer_than_text() {
    let t: [usize; 1] = [1];
    let p: [usize; 2] = [1, 2];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 0] = [];
    assert_eq!(t_check(&r, &e), 0);
}

#[test]
fn test_single_char_pattern() {
    let t: [usize; 5] = [5, 6, 5, 7, 5];
    let p: [usize; 1] = [5];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 3] = [0, 2, 4];
    assert_eq!(t_check(&r, &e), 0);
}

#[test]
fn test_all_distinct_chars() {
    // "abc" all-distinct (mask has one bit per character).
    let t: [usize; 6] = [97, 98, 99, 97, 98, 99];
    let p: [usize; 3] = [97, 98, 99];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 2] = [0, 3];
    assert_eq!(t_check(&r, &e), 0);
}

#[test]
fn test_repeated_chars() {
    // "aab" with repeated 'a' in the mask.
    let t: [usize; 8] = [97, 97, 98, 97, 97, 98, 97, 97];
    let p: [usize; 3] = [97, 97, 98];
    let r = string_myers_bits::shift_and(&t, &p);
    let e: [usize; 2] = [0, 3];
    assert_eq!(t_check(&r, &e), 0);
}
