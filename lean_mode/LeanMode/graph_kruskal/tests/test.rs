/// Helper — check that two f64 values are equal within tolerance.
fn assert_approx_eq(actual: f64, expected: f64, tol: f64) {
    let diff = actual - expected;
    if diff < 0.0 {
        if 0.0 - diff > tol {
            panic!("assertion failed");
        }
    } else if diff > tol {
        panic!("assertion failed");
    }
}

/// Helper — get the total weight of an MST.
fn total_weight(edges: &[(usize, usize, f64)]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: usize = 0;
    while i < edges.len() {
        sum = sum + edges[i].2;
        i += 1;
    }
    sum
}

/// Helper — check that an edge is present in the MST.
fn contains_edge(edges: &[(usize, usize, f64)], u: usize, v: usize, w: f64) -> bool {
    let mut i: usize = 0;
    while i < edges.len() {
        if edges[i].0 == u && edges[i].1 == v {
            let diff = edges[i].2 - w;
            if diff < 0.0 {
                if 0.0 - diff <= 0.001 {
                    return true;
                }
            } else if diff <= 0.001 {
                return true;
            }
        }
        i += 1;
    }
    false
}

// ============================================================================
// Known-answer tests
// ============================================================================

/// Basic square: 1-2-3-4-1 with cross edges.
/// Edges: (1,2,1), (2,3,2), (3,4,1), (4,1,2), (1,3,2)
/// MST: (1,2,1), (2,3,2), (3,4,1), total weight = 4
#[test]
fn test_kruskal_basic_square() {
    let nodes = [1, 2, 3, 4];
    let edges = [
        (1, 2, 1.0),
        (2, 3, 2.0),
        (3, 4, 1.0),
        (4, 1, 2.0),
        (1, 3, 2.0),
    ];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 3);
    assert_approx_eq(total_weight(&mst), 4.0, 0.001);
    assert!(contains_edge(&mst, 1, 2, 1.0));
    assert!(contains_edge(&mst, 2, 3, 2.0));
    assert!(contains_edge(&mst, 3, 4, 1.0));
}

/// Linear graph: 1-2-3-4, all edges must be in MST.
#[test]
fn test_kruskal_linear() {
    let nodes = [1, 2, 3, 4];
    let edges = [
        (1, 2, 1.0),
        (2, 3, 2.0),
        (3, 4, 3.0),
    ];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 3);
    assert_approx_eq(total_weight(&mst), 6.0, 0.001);
}

/// Complete graph K4 with unique weights.
/// MST: 3 smallest edges connecting all 4 nodes.
#[test]
fn test_kruskal_k4() {
    let nodes = [1, 2, 3, 4];
    let edges = [
        (1, 2, 1.0),
        (1, 3, 2.0),
        (1, 4, 3.0),
        (2, 3, 4.0),
        (2, 4, 5.0),
        (3, 4, 6.0),
    ];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 3);
    assert_approx_eq(total_weight(&mst), 6.0, 0.001);
}

/// K5 with all unique weights. MST = 4 smallest edges.
#[test]
fn test_kruskal_k5() {
    let nodes = [1, 2, 3, 4, 5];
    let edges = [
        (1, 2, 1.0),
        (1, 3, 2.0),
        (1, 4, 3.0),
        (1, 5, 4.0),
        (2, 3, 5.0),
        (2, 4, 6.0),
        (2, 5, 7.0),
        (3, 4, 8.0),
        (3, 5, 9.0),
        (4, 5, 10.0),
    ];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 4);
    assert_approx_eq(total_weight(&mst), 10.0, 0.001);
}

// ============================================================================
// Edge case tests
// ============================================================================

/// Empty graph — no nodes, no edges.
#[test]
fn test_empty_graph() {
    let nodes: [usize; 0] = [];
    let edges: [(usize, usize, f64); 0] = [];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 0);
}

/// Single node, no edges.
#[test]
fn test_single_node() {
    let nodes = [1];
    let edges: [(usize, usize, f64); 0] = [];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 0);
}

/// Two nodes, one edge.
#[test]
fn test_single_edge() {
    let nodes = [1, 2];
    let edges = [(1, 2, 5.0)];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 1);
    assert_approx_eq(total_weight(&mst), 5.0, 0.001);
}

/// All nodes isolated — no edges.
#[test]
fn test_isolated_nodes() {
    let nodes = [1, 2, 3, 4];
    let edges: [(usize, usize, f64); 0] = [];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 0);
}

/// Duplicate edges — should pick the smaller weight.
#[test]
fn test_duplicate_edges() {
    let nodes = [1, 2];
    let edges = [
        (1, 2, 5.0),
        (1, 2, 1.0),
    ];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 1);
    assert_approx_eq(total_weight(&mst), 1.0, 0.001);
}

/// Self-loops should be ignored (they don't connect different components).
#[test]
fn test_self_loop() {
    let nodes = [1, 2];
    let edges = [
        (1, 1, 0.5),
        (1, 2, 3.0),
    ];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 1);
    assert_approx_eq(total_weight(&mst), 3.0, 0.001);
}

/// Zero-weight edges are valid.
#[test]
fn test_zero_weight() {
    let nodes = [1, 2, 3];
    let edges = [
        (1, 2, 0.0),
        (2, 3, 0.0),
        (1, 3, 1.0),
    ];
    let mst = graph_kruskal::kruskal(&nodes, &edges);
    assert!(mst.len() == 2);
    assert_approx_eq(total_weight(&mst), 0.0, 0.001);
}
