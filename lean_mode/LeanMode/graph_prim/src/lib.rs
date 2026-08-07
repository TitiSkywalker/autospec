//! Prim's Minimum Spanning Tree algorithm.
//!
//! Starts from an arbitrary node and repeatedly adds the smallest-weight
//! edge that connects a visited node to an unvisited node.
//! O(V·E) time using edge-list scanning.

/// Compute the MST using Prim's algorithm.
///
/// * `nodes` - all node identifiers in the graph
/// * `edges` - edges as (from, to, weight)
///
/// Returns the edges forming the MST.
pub fn prim(nodes: &[usize], edges: &[(usize, usize, f64)]) -> Vec<(usize, usize, f64)> {
    let n_nodes = nodes.len();
    if n_nodes == 0 {
        return Vec::new();
    }

    // Find the maximum node id to size the visited array
    let mut max_node = nodes[0];
    let mut i: usize = 0;
    while i < n_nodes {
        if nodes[i] > max_node {
            max_node = nodes[i];
        }
        i += 1;
    }

    // Initialize visited array — all false
    let vis_size = max_node + 1;
    let mut visited: Vec<bool> = Vec::new();
    let mut j: usize = 0;
    while j < vis_size {
        visited.push(false);
        j += 1;
    }

    // Mark the first node as visited (starting point)
    let start = nodes[0];
    visited[start] = true;

    let mut mst: Vec<(usize, usize, f64)> = Vec::new();
    let edges_len = edges.len();

    // Add |V| - 1 edges
    let mut count: usize = 0;
    while count + 1 < n_nodes {
        let mut found = false;
        let mut min_u: usize = 0;
        let mut min_v: usize = 0;
        let mut min_w: f64 = 0.0;

        // Scan all edges for the minimum crossing edge
        let mut k: usize = 0;
        while k < edges_len {
            let u = edges[k].0;
            let v = edges[k].1;
            let w = edges[k].2;
            if visited[u] && !visited[v] {
                if !found {
                    min_u = u;
                    min_v = v;
                    min_w = w;
                    found = true;
                } else if w < min_w {
                    min_u = u;
                    min_v = v;
                    min_w = w;
                }
            } else if !visited[u] && visited[v] {
                if !found {
                    min_u = v;
                    min_v = u;
                    min_w = w;
                    found = true;
                } else if w < min_w {
                    min_u = v;
                    min_v = u;
                    min_w = w;
                }
            }
            k += 1;
        }

        if found {
            mst.push((min_u, min_v, min_w));
            visited[min_v] = true;
        } else {
            // Graph is disconnected — stop
            count = n_nodes;
        }

        count += 1;
    }

    mst
}
