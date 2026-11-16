// Tests for the `K5`/`K3,3`-minor based planarity test.
//
// Every graph is given as a flat, row-major, symmetric `n × n` 0/1 adjacency
// matrix (zero diagonal).  The expected answers are the classical ones
// (Kuratowski): `K5` and `K3,3` are non-planar, every graph obtained from them
// by deleting an edge or a vertex is planar, and every graph containing a
// subdivision of one of them is non-planar.
//
// The helper names are prefixed `gp_` so they can never collide with (nor be a
// prefix of) any function in `src/lib.rs`.

/// Test-local flat index for a row-major `n × n` matrix.
fn gp_at(adj: &[usize], n: usize, u: usize, v: usize) -> usize {
    let t = u * n;
    let r = t + v;
    adj[r]
}

/// Symmetric graph on `n` vertices with the given unordered edge list.
fn gp_build(n: usize, edges: &[usize]) -> Vec<usize> {
    let mut adj: Vec<usize> = vec![0usize; n * n];
    let m = edges.len() / 2;
    let mut e: usize = 0;
    while e < m {
        let u = edges[2 * e];
        let v = edges[2 * e + 1];
        adj[u * n + v] = 1;
        adj[v * n + u] = 1;
        e += 1;
    }
    adj
}

#[test]
fn test_empty_and_small() {
    // One vertex, no edges: planar.
    let adj1 = gp_build(1, &[]);
    let r1 = graph_planarity::is_planar(1, &adj1);
    assert_eq!(r1, true);

    // Triangle K3: planar.
    let adj3 = gp_build(3, &[0, 1, 1, 2, 2, 0]);
    let r3 = graph_planarity::is_planar(3, &adj3);
    assert_eq!(r3, true);

    // K4: planar (the tetrahedron).
    let adj4 = gp_build(4, &[0, 1, 0, 2, 0, 3, 1, 2, 1, 3, 2, 3]);
    let r4 = graph_planarity::is_planar(4, &adj4);
    assert_eq!(r4, true);
}

#[test]
fn test_k5_is_not_planar() {
    // K5: the first of the two Kuratowski obstructions.
    let adj = gp_build(5, &[0, 1, 0, 2, 0, 3, 0, 4, 1, 2, 1, 3, 1, 4, 2, 3, 2, 4, 3, 4]);
    let r = graph_planarity::is_planar(5, &adj);
    assert_eq!(r, false);
}

#[test]
fn test_k5_minus_edge_is_planar() {
    // Deleting one edge of K5 makes it planar: K5 is edge-minimal
    // non-planar.
    let adj = gp_build(5, &[0, 2, 0, 3, 0, 4, 1, 2, 1, 3, 1, 4, 2, 3, 2, 4, 3, 4]);
    let r = graph_planarity::is_planar(5, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_k33_is_not_planar() {
    // K3,3: the second Kuratowski obstruction.
    let adj = gp_build(6, &[0, 3, 0, 4, 0, 5, 1, 3, 1, 4, 1, 5, 2, 3, 2, 4, 2, 5]);
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, false);
}

#[test]
fn test_k33_minus_edge_is_planar() {
    // K3,3 minus one edge is planar.
    let adj = gp_build(6, &[0, 4, 0, 5, 1, 3, 1, 4, 1, 5, 2, 3, 2, 4, 2, 5]);
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_k5_subdivision_is_not_planar() {
    // Subdivide the edge {0, 1} of K5 using the new vertex 5: the result has
    // K5 as a minor, so it is non-planar.
    let adj = gp_build(
        6,
        &[0, 2, 0, 3, 0, 4, 0, 5, 5, 1, 1, 2, 1, 3, 1, 4, 2, 3, 2, 4, 3, 4],
    );
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, false);
}

#[test]
fn test_k5_plus_isolated_is_not_planar() {
    // K5 together with an isolated vertex: still non-planar.
    let adj = gp_build(6, &[0, 1, 0, 2, 0, 3, 0, 4, 1, 2, 1, 3, 1, 4, 2, 3, 2, 4, 3, 4]);
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, false);
}

#[test]
fn test_cycle_c6_is_planar() {
    let adj = gp_build(6, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 0]);
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_wheel_w5_is_planar() {
    // Wheel on 6 vertices: the hub 5 joined to the 5-cycle 0-1-2-3-4-0.
    let adj = gp_build(
        6,
        &[0, 1, 1, 2, 2, 3, 3, 4, 4, 0, 5, 0, 5, 1, 5, 2, 5, 3, 5, 4],
    );
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_octahedron_is_planar() {
    // Octahedron: the complete tripartite graph K_{2,2,2} on parts
    // {0,1}, {2,3}, {4,5}; octahedra are planar and 4-connected.
    let adj = gp_build(
        6,
        &[
            0, 2, 0, 3, 0, 4, 0, 5, 1, 2, 1, 3, 1, 4, 1, 5, 2, 4, 2, 5, 3, 4, 3, 5,
        ],
    );
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_planarity_is_symmetric_encoding_consistent() {
    // The test-local reader must see exactly the entries the algorithm uses.
    let adj = gp_build(5, &[0, 1, 1, 2, 2, 3, 3, 4, 4, 0]);
    let a = gp_at(&adj, 5, 0, 1);
    let b = gp_at(&adj, 5, 1, 0);
    assert_eq!(a, 1);
    assert_eq!(b, 1);
    let r = graph_planarity::is_planar(5, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_k23_is_planar() {
    // K_{2,3}: bipartite, so no K5 minor, and too small for a K3,3 minor.
    let adj = gp_build(5, &[0, 2, 0, 3, 0, 4, 1, 2, 1, 3, 1, 4]);
    let r = graph_planarity::is_planar(5, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_triangular_prism_is_planar() {
    // Two triangles joined by a perfect matching: planar.
    let adj = gp_build(6, &[0, 1, 1, 2, 2, 0, 3, 4, 4, 5, 5, 3, 0, 3, 1, 4, 2, 5]);
    let r = graph_planarity::is_planar(6, &adj);
    assert_eq!(r, true);
}

#[test]
fn test_k33_with_subdivided_edge_is_not_planar() {
    // K3,3 with the edge {0, 3} subdivided by the new vertex 6: contracting
    // the path back gives K3,3 as a minor, so it is non-planar.
    let adj = gp_build(
        7,
        &[0, 4, 0, 5, 0, 6, 6, 3, 1, 3, 1, 4, 1, 5, 2, 3, 2, 4, 2, 5],
    );
    let r = graph_planarity::is_planar(7, &adj);
    assert_eq!(r, false);
}

#[test]
fn test_k33_plus_isolated_is_not_planar() {
    let adj = gp_build(
        7,
        &[0, 3, 0, 4, 0, 5, 1, 3, 1, 4, 1, 5, 2, 3, 2, 4, 2, 5],
    );
    let r = graph_planarity::is_planar(7, &adj);
    assert_eq!(r, false);
}

#[test]
fn test_cube_is_planar() {
    // The cube graph Q3: two squares joined by a matching, bipartite, with
    // n = 8 > 6 so the K3,3 search has to reject the extra vertices.
    let adj = gp_build(
        8,
        &[
            0, 1, 1, 2, 2, 3, 3, 0, 4, 5, 5, 6, 6, 7, 7, 4, 0, 4, 1, 5, 2, 6, 3, 7,
        ],
    );
    let r = graph_planarity::is_planar(8, &adj);
    assert_eq!(r, true);
}
