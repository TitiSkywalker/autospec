/// Helper — check approx equality of two f64 values.
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

/// Get the total weight of an edge list.
fn total_weight(edges: &[(usize, usize, f64)]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: usize = 0;
    while i < edges.len() {
        sum = sum + edges[i].2;
        i += 1;
    }
    sum
}

// ============================================================================
// Basic tests
// ============================================================================

/// Square graph: 4 nodes, 5 edges. MST weight = 4.
#[test]
fn test_prim_square() {
    let nodes = [1, 2, 3, 4];
    let edges = [
        (1, 2, 1.0),
        (2, 3, 2.0),
        (3, 4, 1.0),
        (4, 1, 2.0),
        (1, 3, 2.0),
    ];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 3);
    assert_approx_eq(total_weight(&mst), 4.0, 0.001);
}

/// Linear graph: 4 nodes in a line.
#[test]
fn test_prim_linear() {
    let nodes = [1, 2, 3, 4];
    let edges = [
        (1, 2, 1.0),
        (2, 3, 2.0),
        (3, 4, 3.0),
    ];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 3);
    assert_approx_eq(total_weight(&mst), 6.0, 0.001);
}

/// Complete graph K4 with unique weights. MST weight = 6.
#[test]
fn test_prim_k4() {
    let nodes = [1, 2, 3, 4];
    let edges = [
        (1, 2, 1.0),
        (1, 3, 2.0),
        (1, 4, 3.0),
        (2, 3, 4.0),
        (2, 4, 5.0),
        (3, 4, 6.0),
    ];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 3);
    assert_approx_eq(total_weight(&mst), 6.0, 0.001);
}

/// K5 with unique weights. MST weight = 10.
#[test]
fn test_prim_k5() {
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
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 4);
    assert_approx_eq(total_weight(&mst), 10.0, 0.001);
}

// ============================================================================
// Edge case tests
// ============================================================================

/// Empty graph.
#[test]
fn test_empty_graph() {
    let nodes: [usize; 0] = [];
    let edges: [(usize, usize, f64); 0] = [];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 0);
}

/// Single node, no edges.
#[test]
fn test_single_node() {
    let nodes = [1];
    let edges: [(usize, usize, f64); 0] = [];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 0);
}

/// Two nodes, one edge.
#[test]
fn test_single_edge() {
    let nodes = [1, 2];
    let edges = [(1, 2, 5.0)];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 1);
    assert_approx_eq(total_weight(&mst), 5.0, 0.001);
}

/// All nodes isolated.
#[test]
fn test_isolated_nodes() {
    let nodes = [1, 2, 3, 4];
    let edges: [(usize, usize, f64); 0] = [];
    let mst = prim::prim(&nodes, &edges);
    // No reachable edges → only start node is visited
    assert!(mst.len() == 0);
}

/// Duplicate edges — picks the smaller weight.
#[test]
fn test_duplicate_edges() {
    let nodes = [1, 2];
    let edges = [
        (1, 2, 5.0),
        (1, 2, 1.0),
    ];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 1);
    assert_approx_eq(total_weight(&mst), 1.0, 0.001);
}

/// Self-loops should be ignored.
#[test]
fn test_self_loop() {
    let nodes = [1, 2];
    let edges = [
        (1, 1, 0.5),
        (1, 2, 3.0),
    ];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 1);
    assert_approx_eq(total_weight(&mst), 3.0, 0.001);
}

/// Zero-weight edges.
#[test]
fn test_zero_weight() {
    let nodes = [1, 2, 3];
    let edges = [
        (1, 2, 0.0),
        (2, 3, 0.0),
        (1, 3, 1.0),
    ];
    let mst = prim::prim(&nodes, &edges);
    assert!(mst.len() == 2);
    assert_approx_eq(total_weight(&mst), 0.0, 0.001);
}
