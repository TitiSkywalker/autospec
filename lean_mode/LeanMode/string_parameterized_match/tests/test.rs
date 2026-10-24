//! Small, hand-checkable tests for parameterized matching.
//!
//! Characters are `usize`-encoded; the occurrence lists are checked element by
//! element (AENEAS has no `PartialEq` on `Vec`).

#[test]
fn test_prev_encoding_basic() {
    let s: [usize; 3] = [1, 2, 1];
    let e = string_parameterized_match::prev_encoding(&s);
    assert_eq!(e.len(), 3);
    assert_eq!(e[0], 0);
    assert_eq!(e[1], 0);
    assert_eq!(e[2], 2); // the 1 at index 2 points back to index 0
}

#[test]
fn test_prev_encoding_repeats() {
    let s: [usize; 5] = [1, 1, 1, 2, 1];
    let e = string_parameterized_match::prev_encoding(&s);
    assert_eq!(e.len(), 5);
    assert_eq!(e[0], 0);
    assert_eq!(e[1], 1);
    assert_eq!(e[2], 1);
    assert_eq!(e[3], 0); // 2 is new
    assert_eq!(e[4], 2); // the 1 at index 4 points back to index 2
}

#[test]
fn test_pmatch_basic() {
    // p = a b a, t = c d c e f e  ->  matches at 0 ([c d c]) and 3 ([e f e]).
    let t: [usize; 6] = [3, 4, 3, 5, 6, 5];
    let p: [usize; 3] = [1, 2, 1];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0], 0);
    assert_eq!(r[1], 3);
}

#[test]
fn test_pmatch_alphabets_of_size_one() {
    // p = x x x, t = x x x x x  ->  matches at 0, 1, 2.
    let t: [usize; 5] = [7, 7, 7, 7, 7];
    let p: [usize; 3] = [7, 7, 7];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 3);
    assert_eq!(r[0], 0);
    assert_eq!(r[1], 1);
    assert_eq!(r[2], 2);
}

#[test]
fn test_pmatch_all_distinct_windows() {
    // p = a b c (all distinct), t = a b c d  ->  windows [a b c] and [b c d].
    let t: [usize; 4] = [1, 2, 3, 4];
    let p: [usize; 3] = [1, 2, 3];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0], 0);
    assert_eq!(r[1], 1);
}

#[test]
fn test_pmatch_repeated_pattern() {
    // p = a a, t = b c c d d  ->  matches the windows [c c] and [d d].
    let t: [usize; 5] = [2, 3, 3, 4, 4];
    let p: [usize; 2] = [1, 1];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0], 1);
    assert_eq!(r[1], 3);
}

#[test]
fn test_pmatch_window_reset() {
    // p = a b (distinct).  t = 7 0 7.  The window [0 7] at i = 1 has 7 already
    // seen *before* the window — window-local encoding still calls it new, so
    // the window matches; both i = 0 and i = 1 match.
    let t: [usize; 3] = [7, 0, 7];
    let p: [usize; 2] = [9, 8];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 2);
    assert_eq!(r[0], 0);
    assert_eq!(r[1], 1);
}

#[test]
fn test_pmatch_no_match() {
    // p = a a b (prev encoding [0,1,0]) has no match in t = 5 6 5 7.
    let t: [usize; 4] = [5, 6, 5, 7];
    let p: [usize; 3] = [1, 1, 2];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 0);
}

#[test]
fn test_pmatch_pattern_longer_than_text() {
    let t: [usize; 2] = [1, 2];
    let p: [usize; 3] = [1, 2, 3];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 0);
}

#[test]
fn test_pmatch_empty_pattern() {
    // An empty pattern p-matches nowhere under this definition.
    let t: [usize; 3] = [1, 2, 3];
    let p: [usize; 0] = [];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 0);
}

#[test]
fn test_pmatch_exact() {
    let t: [usize; 3] = [4, 5, 6];
    let p: [usize; 3] = [4, 5, 6];
    let r = string_parameterized_match::pmatch(&t, &p);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0], 0);
}
