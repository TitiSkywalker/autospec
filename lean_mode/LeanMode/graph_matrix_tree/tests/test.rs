#[test]
fn test_single_vertex() {
    // n=1: trivially one spanning tree (empty).
    let adj = vec![0];
    let result = graph_matrix_tree::matrix_tree(1, &adj);
    assert_eq!(result, 1);
}

#[test]
fn test_single_edge() {
    // 0 — 1: one spanning tree (the single edge).
    let adj = vec![0, 1, 1, 0];
    let result = graph_matrix_tree::matrix_tree(2, &adj);
    assert_eq!(result, 1);
}

#[test]
fn test_triangle() {
    // K3: three spanning trees (each drops one of the three edges).
    let adj = vec![
        0, 1, 1,
        1, 0, 1,
        1, 1, 0,
    ];
    let result = graph_matrix_tree::matrix_tree(3, &adj);
    assert_eq!(result, 3);
}

#[test]
fn test_cycle4() {
    // A 4-cycle has 4 spanning trees (drop any one edge).
    let adj = vec![
        0, 1, 0, 1,
        1, 0, 1, 0,
        0, 1, 0, 1,
        1, 0, 1, 0,
    ];
    let result = graph_matrix_tree::matrix_tree(4, &adj);
    assert_eq!(result, 4);
}

#[test]
fn test_complete_k4() {
    // K4 has 4^(4-2) = 16 spanning trees (Cayley's formula).
    let adj = vec![
        0, 1, 1, 1,
        1, 0, 1, 1,
        1, 1, 0, 1,
        1, 1, 1, 0,
    ];
    let result = graph_matrix_tree::matrix_tree(4, &adj);
    assert_eq!(result, 16);
}

#[test]
fn test_disconnected() {
    // Two vertices, no edge: 0 spanning trees.
    let adj = vec![0, 0, 0, 0];
    let result = graph_matrix_tree::matrix_tree(2, &adj);
    assert_eq!(result, 0);
}
