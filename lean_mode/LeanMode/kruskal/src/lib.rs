//! Kruskal's Minimum Spanning Tree algorithm.
//!
//! Uses union-find with path compression to find the MST of a weighted
//! undirected graph.

/// Find the root of a node (iterative, no path compression).
pub fn find(parent: &[usize], x: usize) -> usize {
    let mut current = x;
    while parent[current] != current {
        current = parent[current];
    }
    current
}

/// Union two sets by attaching one root to another.
pub fn union(parent: &mut [usize], x: usize, y: usize) {
    let x_root = find(parent, x);
    let y_root = find(parent, y);
    if x_root != y_root {
        parent[x_root] = y_root;
    }
}

/// Compute the MST using Kruskal's algorithm.
///
/// * `nodes` - all node identifiers in the graph
/// * `edges` - edges as (from, to, weight)
///
/// Returns the edges forming the MST.
pub fn kruskal(nodes: &[usize], edges: &[(usize, usize, f64)]) -> Vec<(usize, usize, f64)> {
    let n_nodes = nodes.len();
    if n_nodes == 0 {
        return Vec::new();
    }

    // Find the maximum node id to size the parent array
    let mut max_node = nodes[0];
    let mut i: usize = 0;
    while i < n_nodes {
        if nodes[i] > max_node {
            max_node = nodes[i];
        }
        i += 1;
    }

    // Initialize union-find: each node is its own parent
    let parent_size = max_node + 1;
    let mut parent: Vec<usize> = Vec::new();
    let mut j: usize = 0;
    while j < parent_size {
        parent.push(j);
        j += 1;
    }
    i = 0;
    while i < n_nodes {
        parent[nodes[i]] = nodes[i];
        i += 1;
    }

    // Sort edges by weight (bubble sort)
    let edges_len = edges.len();
    let mut sorted_edges: Vec<(usize, usize, f64)> = Vec::new();
    let mut k: usize = 0;
    while k < edges_len {
        sorted_edges.push(edges[k]);
        k += 1;
    }

    k = 0;
    while k < edges_len {
        let mut m: usize = 0;
        while m + 1 < edges_len - k {
            let a_weight = sorted_edges[m].2;
            let b_weight = sorted_edges[m + 1].2;
            if a_weight > b_weight {
                let a = sorted_edges[m];
                let b = sorted_edges[m + 1];
                sorted_edges[m] = b;
                sorted_edges[m + 1] = a;
            }
            m += 1;
        }
        k += 1;
    }

    // Build MST: greedily add smallest edges that don't form cycles
    let mut mst: Vec<(usize, usize, f64)> = Vec::new();
    k = 0;
    while k < edges_len {
        let u = sorted_edges[k].0;
        let v = sorted_edges[k].1;
        let w = sorted_edges[k].2;
        if find(&parent, u) != find(&parent, v) {
            union(&mut parent, u, v);
            mst.push((u, v, w));
        }
        k += 1;
    }
    mst
}
