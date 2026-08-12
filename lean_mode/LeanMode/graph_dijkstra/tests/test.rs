#[test]
fn test_simple() {
    let from = vec![0, 0, 1];
    let to = vec![1, 2, 2];
    let w = vec![2, 6, 3];
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 2);
    assert_eq!(dist[2], 5);
    assert!(reachable[0]);
    assert!(reachable[1]);
    assert!(reachable[2]);
}

#[test]
fn test_unreachable() {
    let from = vec![0];
    let to = vec![1];
    let w = vec![1];
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert!(reachable[0]);
    assert!(reachable[1]);
    assert!(!reachable[2]); // node 2 is unreachable
}

#[test]
fn test_single_node() {
    let (dist, reachable) = graph_dijkstra::dijkstra(1, &[], &[], &[], 0);
    assert_eq!(dist[0], 0);
    assert!(reachable[0]);
}

#[test]
fn test_directed() {
    let from = vec![0];
    let to = vec![1];
    let w = vec![1];
    let (dist, reachable) = graph_dijkstra::dijkstra(2, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert!(reachable[0]);
    assert!(reachable[1]);
}

#[test]
fn test_start_from_middle() {
    let from = vec![0, 1, 0];
    let to = vec![1, 2, 2];
    let w = vec![4, 5, 10];
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &from, &to, &w, 1);
    assert_eq!(dist[1], 0);
    assert_eq!(dist[2], 5);
    assert!(reachable[1]);
    assert!(reachable[2]);
    assert!(!reachable[0]); // node 0 cannot be reached when starting from 1 (directed graph)
}

// ── From petgraph (adapted f64→i64) ─────────────────────────────────────────

#[test]
fn test_simple_graph_4node() {
    // 0--1--1  0--3--3, 1--2--2, 1--3--7, 3--1--4, 3--2--6
    let from = vec![0, 0, 1, 1, 3, 3];
    let to   = vec![1, 3, 2, 3, 1, 2];
    let w    = vec![1, 3, 2, 7, 4, 6];
    let (dist, reachable) = graph_dijkstra::dijkstra(4, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], 3);
    assert_eq!(dist[3], 3);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_petgraph_example_8node() {
    // all weights=1, directed cycle graph
    let from = vec![0, 1, 1, 2, 3, 4, 5, 6, 7];
    let to   = vec![1, 2, 4, 3, 0, 5, 6, 7, 4];
    let w    = vec![1, 1, 1, 1, 1, 1, 1, 1, 1];
    let (dist, reachable) = graph_dijkstra::dijkstra(8, &from, &to, &w, 1);
    assert_eq!(dist[0], 3);
    assert_eq!(dist[1], 0);
    assert_eq!(dist[2], 1);
    assert_eq!(dist[3], 2);
    assert_eq!(dist[4], 1);
    assert_eq!(dist[5], 2);
    assert_eq!(dist[6], 3);
    assert_eq!(dist[7], 4);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_empty_graph() {
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &[], &[], &[], 0);
    assert_eq!(dist[0], 0);
    assert!(reachable[0]);
    assert!(!reachable[1]);
    assert!(!reachable[2]);
}

#[test]
fn test_disconnected() {
    let from = vec![0, 2];
    let to   = vec![1, 3];
    let w    = vec![1, 1];
    let (dist, reachable) = graph_dijkstra::dijkstra(4, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert!(reachable[0]);
    assert!(reachable[1]);
    assert!(!reachable[2]); // no path from 0 to 2
    assert!(!reachable[3]); // no path from 0 to 3 (edge 2→3 exists but 2 is unreachable)
}

#[test]
fn test_indirect_better() {
    // 0→1(2), 0→2(4), 1→2(1). 0→2 via 1 is 3, not 4
    let from = vec![0, 0, 1];
    let to   = vec![1, 2, 2];
    let w    = vec![2, 4, 1];
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 2);
    assert_eq!(dist[2], 3);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_start_zero() {
    let from = vec![0, 1];
    let to   = vec![1, 2];
    let w    = vec![5, 3];
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert!(reachable[0]);
}

#[test]
fn test_all_reachable() {
    let from = vec![0, 1];
    let to   = vec![1, 2];
    let w    = vec![1, 1];
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &from, &to, &w, 0);
    assert_eq!(dist.len(), 3);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], 2);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_linear_chain() {
    // 0→1(2), 1→2(3), 2→3(1)
    let from = vec![0, 1, 2];
    let to   = vec![1, 2, 3];
    let w    = vec![2, 3, 1];
    let (dist, reachable) = graph_dijkstra::dijkstra(4, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 2);
    assert_eq!(dist[2], 5);
    assert_eq!(dist[3], 6);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_complete_k3() {
    let from = vec![0, 0, 1, 1, 2, 2];
    let to   = vec![1, 2, 0, 2, 0, 1];
    let w    = vec![1, 1, 1, 1, 1, 1];
    let (dist, reachable) = graph_dijkstra::dijkstra(3, &from, &to, &w, 0);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], 1);
    assert!(reachable.iter().all(|&r| r));
}
