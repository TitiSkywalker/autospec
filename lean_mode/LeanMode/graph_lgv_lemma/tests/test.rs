#[test]
fn test_single_path_diamond() {
    // One source, one sink, two parallel paths: e = 2, det = 2.
    // 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3.
    let adj = vec![
        0, 1, 1, 0,
        0, 0, 0, 1,
        0, 0, 0, 1,
        0, 0, 0, 0,
    ];
    let src = vec![0usize];
    let snk = vec![3usize];
    let result = graph_lgv_lemma::graph_lgv_lemma(4, &adj, 1, &src, &snk);
    assert_eq!(result, 2);
}

#[test]
fn test_two_sources_cancel() {
    // Two sources, two sinks, every path goes through the 2-vertex middle, so
    // no two vertex-disjoint paths exist and the determinant vanishes (the
    // intersecting path families cancel).
    // Edges: 0->2,0->3, 1->2,1->3, 2->4,2->5, 3->4,3->5.
    let adj = vec![
        0, 0, 1, 1, 0, 0,
        0, 0, 1, 1, 0, 0,
        0, 0, 0, 0, 1, 1,
        0, 0, 0, 0, 1, 1,
        0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];
    let src = vec![0usize, 1usize];
    let snk = vec![4usize, 5usize];
    let result = graph_lgv_lemma::graph_lgv_lemma(6, &adj, 2, &src, &snk);
    assert_eq!(result, 0);
}

#[test]
fn test_two_sources_disjoint() {
    // Edges: 0->2, 0->3, 1->3, 2->4, 3->4, 3->5.
    // e = [[2, 1], [1, 1]]; det = 2*1 - 1*1 = 1 (one disjoint family).
    let adj = vec![
        0, 0, 1, 1, 0, 0,
        0, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 1, 0,
        0, 0, 0, 0, 1, 1,
        0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];
    let src = vec![0usize, 1usize];
    let snk = vec![4usize, 5usize];
    let result = graph_lgv_lemma::graph_lgv_lemma(6, &adj, 2, &src, &snk);
    assert_eq!(result, 1);
}
