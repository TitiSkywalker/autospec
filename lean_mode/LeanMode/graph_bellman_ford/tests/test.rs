#[test]
fn test_simple() {
    // Same graph as dijkstra test_simple, but Bellman-Ford also works here
    let from = vec![0, 0, 1];
    let to = vec![1, 2, 2];
    let w = vec![2, 6, 3];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
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
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert!(reachable[0]);
    assert!(reachable[1]);
    assert!(!reachable[2]);
}

#[test]
fn test_single_node() {
    let result = graph_bellman_ford::bellman_ford(1, &[], &[], &[], 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert!(reachable[0]);
}

#[test]
fn test_directed() {
    let from = vec![0];
    let to = vec![1];
    let w = vec![1];
    let result = graph_bellman_ford::bellman_ford(2, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
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
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 1);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[1], 0);
    assert_eq!(dist[2], 5);
    assert!(reachable[1]);
    assert!(reachable[2]);
    assert!(!reachable[0]);
}

// ── Negative weight tests (unique to Bellman-Ford) ──────────────────────

#[test]
fn test_negative_weights_no_cycle() {
    // 0→1(1), 0→2(4), 1→2(-2), 2→3(1)
    // Shortest: 0→2 via 1 = 1 + (-2) = -1, 0→3 = -1 + 1 = 0
    let from = vec![0, 0, 1, 2];
    let to = vec![1, 2, 2, 3];
    let w = vec![1, 4, -2, 1];
    let result = graph_bellman_ford::bellman_ford(4, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], -1);
    assert_eq!(dist[3], 0);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_negative_cycle_detection() {
    // 0→1(-1), 1→2(-1), 2→0(-1) — negative cycle
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 0];
    let w = vec![-1, -1, -1];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    assert!(!result.is_ok());
}

#[test]
fn test_negative_cycle_small() {
    // 2-node cycle: 0→1(-5), 1→0(-3)
    let from = vec![0, 1];
    let to = vec![1, 0];
    let w = vec![-5, -3];
    let result = graph_bellman_ford::bellman_ford(2, &from, &to, &w, 0);
    assert!(!result.is_ok());
}

#[test]
fn test_negative_weights_chain() {
    // 0→1(2), 1→2(-3), 2→3(1) — simple chain with one negative
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 3];
    let w = vec![2, -3, 1];
    let result = graph_bellman_ford::bellman_ford(4, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 2);
    assert_eq!(dist[2], -1);
    assert_eq!(dist[3], 0);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_no_negative_cycle_with_positive_cycle() {
    // 0→1(1), 1→2(2), 2→0(5) — positive cycle, no negative cycle
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 0];
    let w = vec![1, 2, 5];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], 3);
    assert!(reachable.iter().all(|&r| r));
}

// ── From graph_dijkstra tests (adapted) ──────────────────────────────────

#[test]
fn test_simple_graph_4node() {
    // Same as dijkstra test: 0--1--1, 0--3--3, 1--2--2, 1--3--7, 3--1--4, 3--2--6
    let from = vec![0, 0, 1, 1, 3, 3];
    let to = vec![1, 3, 2, 3, 1, 2];
    let w = vec![1, 3, 2, 7, 4, 6];
    let result = graph_bellman_ford::bellman_ford(4, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], 3);
    assert_eq!(dist[3], 3);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_empty_graph() {
    let result = graph_bellman_ford::bellman_ford(3, &[], &[], &[], 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert!(reachable[0]);
    assert!(!reachable[1]);
    assert!(!reachable[2]);
}

#[test]
fn test_disconnected() {
    let from = vec![0, 2];
    let to = vec![1, 3];
    let w = vec![1, 1];
    let result = graph_bellman_ford::bellman_ford(4, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert!(reachable[0]);
    assert!(reachable[1]);
    assert!(!reachable[2]);
    assert!(!reachable[3]);
}

#[test]
fn test_indirect_better() {
    // 0→1(2), 0→2(4), 1→2(1). 0→2 via 1 is 3, not 4
    let from = vec![0, 0, 1];
    let to = vec![1, 2, 2];
    let w = vec![2, 4, 1];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 2);
    assert_eq!(dist[2], 3);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_linear_chain() {
    // 0→1(2), 1→2(3), 2→3(1)
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 3];
    let w = vec![2, 3, 1];
    let result = graph_bellman_ford::bellman_ford(4, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 2);
    assert_eq!(dist[2], 5);
    assert_eq!(dist[3], 6);
    assert!(reachable.iter().all(|&r| r));
}

#[test]
fn test_complete_k3() {
    let from = vec![0, 0, 1, 1, 2, 2];
    let to = vec![1, 2, 0, 2, 0, 1];
    let w = vec![1, 1, 1, 1, 1, 1];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], 1);
    assert!(reachable.iter().all(|&r| r));
}

// ── Additional edge cases ───────────────────────────────────────────────

#[test]
fn test_disconnected_negative_cycle() {
    // Negative cycle on nodes 0↔1, but start=2 is disconnected
    // The negative cycle should still be detected
    let from = vec![0, 1];
    let to = vec![1, 0];
    let w = vec![-1, -1];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 2);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert!(reachable[2]);
    assert_eq!(dist[2], 0);
    assert!(!reachable[0]);
    assert!(!reachable[1]);
}

#[test]
fn test_start_zero() {
    let from = vec![0, 1];
    let to = vec![1, 2];
    let w = vec![5, 3];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist[0], 0);
    assert!(reachable[0]);
}

#[test]
fn test_all_reachable() {
    let from = vec![0, 1];
    let to = vec![1, 2];
    let w = vec![1, 1];
    let result = graph_bellman_ford::bellman_ford(3, &from, &to, &w, 0);
    let (dist, reachable) = match result {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    assert_eq!(dist.len(), 3);
    assert_eq!(dist[0], 0);
    assert_eq!(dist[1], 1);
    assert_eq!(dist[2], 2);
    assert!(reachable.iter().all(|&r| r));
}
