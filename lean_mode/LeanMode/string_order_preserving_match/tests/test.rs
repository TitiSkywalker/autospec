//! Small, hand-checkable tests (translated and `#eval`'d by CI).
//!
//! All values are `usize`.  `op_match` returns the *positions*; a `Vec` is
//! compared element-by-element (never `assert_eq!` on two `Vec`s).

#[test]
fn test_rank_encoding_distinct() {
    let s: [usize; 3] = [10, 20, 30];
    let r = string_order_preserving_match::rank_encoding(&s);
    assert_eq!(r.len(), 3);
    assert_eq!(r[0], 0);
    assert_eq!(r[1], 1);
    assert_eq!(r[2], 2);
}

#[test]
fn test_rank_encoding_ties() {
    // [5, 5, 3, 5]: rank = [#{<5}, #{<5}, #{<3}, #{<5}] = [1, 1, 0, 1].
    let s: [usize; 4] = [5, 5, 3, 5];
    let r = string_order_preserving_match::rank_encoding(&s);
    assert_eq!(r.len(), 4);
    assert_eq!(r[0], 1);
    assert_eq!(r[1], 1);
    assert_eq!(r[2], 0);
    assert_eq!(r[3], 1);
}

#[test]
fn test_distinct_increasing() {
    // p = [1,2,3] (increasing) matches the increasing length-3 windows.
    let t: [usize; 4] = [10, 20, 30, 40];
    let p: [usize; 3] = [1, 2, 3];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 2);
    assert_eq!(occ[0], 0);
    assert_eq!(occ[1], 1);
}

#[test]
fn test_ties_within_window() {
    // p = [1,1,2] matches [5,5,7] (a tied first pair then a larger element).
    let t: [usize; 4] = [5, 5, 7, 9];
    let p: [usize; 3] = [1, 1, 2];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 1);
    assert_eq!(occ[0], 0);
}

#[test]
fn test_ties_across_boundary() {
    // p = [2,0] is "decreasing".  Window [3,1] matches, [1,3] does not, and the
    // tied value 3 at index 2 (equal to index 0) still yields a match with 2.
    let t: [usize; 4] = [3, 1, 3, 2];
    let p: [usize; 2] = [2, 0];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 2);
    assert_eq!(occ[0], 0);
    assert_eq!(occ[1], 2);
}

#[test]
fn test_order_not_equality() {
    // Order-preserving, not substring: p = [0,1] (increasing) matches every
    // increasing adjacent pair, at positions 1 and 3.
    let t: [usize; 5] = [10, 5, 8, 3, 9];
    let p: [usize; 2] = [0, 1];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 2);
    assert_eq!(occ[0], 1);
    assert_eq!(occ[1], 3);
}

#[test]
fn test_no_match() {
    let t: [usize; 4] = [1, 2, 3, 4];
    let p: [usize; 3] = [3, 2, 1];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 0);
}

#[test]
fn test_decreasing_pattern() {
    // p = [30,20,10] (strictly decreasing) matches every decreasing length-3
    // window of [5,4,3,2,1].
    let t: [usize; 5] = [5, 4, 3, 2, 1];
    let p: [usize; 3] = [30, 20, 10];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 3);
    assert_eq!(occ[0], 0);
    assert_eq!(occ[1], 1);
    assert_eq!(occ[2], 2);
}

#[test]
fn test_single_element_pattern() {
    // A length-1 pattern matches (order-isomorphically) everywhere.
    let t: [usize; 5] = [3, 1, 4, 1, 5];
    let p: [usize; 1] = [7];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 5);
    assert_eq!(occ[0], 0);
    assert_eq!(occ[1], 1);
    assert_eq!(occ[2], 2);
    assert_eq!(occ[3], 3);
    assert_eq!(occ[4], 4);
}

#[test]
fn test_pattern_longer_than_text() {
    let t: [usize; 2] = [1, 2];
    let p: [usize; 3] = [1, 2, 3];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 0);
}

#[test]
fn test_empty_pattern_matches_nothing() {
    // Documented convention: the empty pattern matches nowhere.
    let t: [usize; 3] = [1, 2, 3];
    let p: [usize; 0] = [];
    let occ = string_order_preserving_match::op_match(&t, &p);
    assert_eq!(occ.len(), 0);
}
