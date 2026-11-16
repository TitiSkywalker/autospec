//! Small concrete tests for `max_weight_matching`.
//!
//! Kept tiny and fast on purpose: `tests/test.rs` is translated to
//! `Test_test.lean` and `#eval`'d, and Aeneas' `Vec` has O(length) indexing.
//! The randomized / brute-force differential tests live in the (untranslated)
//! `#[cfg(test)] mod tests` of `src/lib.rs`.

#[test]
fn test_empty() {
    let adj: Vec<i64> = vec![];
    let w: Vec<i64> = vec![];
    let (total, matched) = graph_weighted_matching::max_weight_matching(0, &adj, &w);
    assert_eq!(total, 0);
    assert_eq!(matched.len(), 0);
}

#[test]
fn test_single_edge() {
    let adj = vec![0, 1, 1, 0];
    let w = vec![0, 5, 5, 0];
    let (total, matched) = graph_weighted_matching::max_weight_matching(2, &adj, &w);
    assert_eq!(total, 5);
    assert_eq!(matched[1], 1);
    assert_eq!(matched[2], 1);
}

#[test]
fn test_negative_edge_unused() {
    // A single edge of negative weight: the empty matching (weight 0) is best.
    let adj = vec![0, 1, 1, 0];
    let w = vec![0, -5, -5, 0];
    let (total, matched) = graph_weighted_matching::max_weight_matching(2, &adj, &w);
    assert_eq!(total, 0);
    assert_eq!(matched[1], 0);
    assert_eq!(matched[2], 0);
}

#[test]
fn test_triangle_unit() {
    // Unit triangle: only one edge can be chosen.
    let adj = vec![0, 1, 1, 1, 0, 1, 1, 1, 0];
    let w = vec![0, 1, 1, 1, 0, 1, 1, 1, 0];
    let (total, _matched) = graph_weighted_matching::max_weight_matching(3, &adj, &w);
    assert_eq!(total, 1);
}

#[test]
fn test_path4() {
    // 0-1 (3), 1-2 (2), 2-3 (4): best is {0-1, 2-3} = 7.
    let adj = vec![0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0];
    let w = vec![0, 3, 0, 0, 3, 0, 2, 0, 0, 2, 0, 4, 0, 0, 4, 0];
    let (total, matched) = graph_weighted_matching::max_weight_matching(4, &adj, &w);
    assert_eq!(total, 7);
    assert_eq!(matched[1], 1);
    assert_eq!(matched[11], 1);
}

#[test]
fn test_five_cycle() {
    // 5-cycle with weights 1,2,3,4,5: best is {2-3, 4-0} = 8.
    let adj = vec![
        0, 1, 0, 0, 1,
        1, 0, 1, 0, 0,
        0, 1, 0, 1, 0,
        0, 0, 1, 0, 1,
        1, 0, 0, 1, 0,
    ];
    let w = vec![
        0, 1, 0, 0, 5,
        1, 0, 2, 0, 0,
        0, 2, 0, 3, 0,
        0, 0, 3, 0, 4,
        5, 0, 0, 4, 0,
    ];
    let (total, _matched) = graph_weighted_matching::max_weight_matching(5, &adj, &w);
    assert_eq!(total, 8);
}

#[test]
fn test_star() {
    // Star centred at 0 with weights 1,2,3: best is the single heaviest edge.
    let adj = vec![0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0];
    let w = vec![0, 1, 2, 3, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0];
    let (total, _matched) = graph_weighted_matching::max_weight_matching(4, &adj, &w);
    assert_eq!(total, 3);
}
