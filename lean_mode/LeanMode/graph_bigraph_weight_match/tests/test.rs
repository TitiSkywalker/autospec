#[test]
fn test_single() {
    // n = 1, single edge of weight 7
    let w = vec![7];
    let (weight, matched) = graph_bigraph_weight_match::kuhn_munkres(1, &w);
    assert_eq!(weight, 7);
    assert_eq!(matched[0], 1);
}

#[test]
fn test_2x2_prefer_diagonal() {
    // 2x2: w[0][0]=1, w[0][1]=5, w[1][0]=5, w[1][1]=1.
    // Best perfect matching: 0-1 and 1-0 (weight 10).
    let w = vec![
        1, 5,
        5, 1,
    ];
    let (weight, matched) = graph_bigraph_weight_match::kuhn_munkres(2, &w);
    assert_eq!(weight, 10);
    assert_eq!(matched[1], 1); // 0-1
    assert_eq!(matched[2], 1); // 1-0
}

#[test]
fn test_negative_weights() {
    // 2x2 with negative weights: still a perfect matching must be chosen.
    // w = [[1, -100], [-100, 2]] → best is diagonal (0-0, 1-1) = 3.
    let w = vec![
        1, -100,
        -100, 2,
    ];
    let (weight, _matched) = graph_bigraph_weight_match::kuhn_munkres(2, &w);
    assert_eq!(weight, 3);
}

#[test]
fn test_3x3() {
    // classic assignment: rows are workers, cols are jobs
    // [[4, 1, 3], [2, 0, 5], [3, 2, 2]]
    // optimal: 0->0 (4), 1->2 (5), 2->1 (2) = 11
    let w = vec![
        4, 1, 3,
        2, 0, 5,
        3, 2, 2,
    ];
    let (weight, _matched) = graph_bigraph_weight_match::kuhn_munkres(3, &w);
    assert_eq!(weight, 11);
}

#[test]
fn test_equal_weights() {
    // 3x3 all weights 1 → any perfect matching has weight 3
    let w = vec![1; 9];
    let (weight, matched) = graph_bigraph_weight_match::kuhn_munkres(3, &w);
    assert_eq!(weight, 3);
    // exactly 3 matched edges
    let mut count = 0;
    let mut i = 0;
    while i < 9 {
        count = count + matched[i];
        i += 1;
    }
    assert_eq!(count, 3);
}
