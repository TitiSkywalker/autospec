#[test]
fn test_chain() {
    // 0 → 1 → 2 → 3
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 3];
    let result = graph_bfs::bfs(4, &from, &to, 0, 3);
    let d = match result { Some(x) => x, None => 4 };
    assert_eq!(d, 3);
}

#[test]
fn test_unreachable() {
    // only edge 0 → 1; node 2 is unreachable from 0
    let from = vec![0];
    let to = vec![1];
    let result = graph_bfs::bfs(3, &from, &to, 0, 2);
    let b = match result { Some(_) => false, None => true };
    assert!(b);
}

#[test]
fn test_start_equals_target() {
    let from = vec![0, 1];
    let to = vec![1, 2];
    let result = graph_bfs::bfs(3, &from, &to, 1, 1);
    let d = match result { Some(x) => x, None => 3 };
    assert_eq!(d, 0);
}

#[test]
fn test_diamond() {
    // 0 → 1, 0 → 2, 1 → 3, 2 → 3  →  shortest 0→…→3 is 2 edges
    let from = vec![0, 0, 1, 2];
    let to = vec![1, 2, 3, 3];
    let result = graph_bfs::bfs(4, &from, &to, 0, 3);
    let d = match result { Some(x) => x, None => 4 };
    assert_eq!(d, 2);
}

#[test]
fn test_disconnected() {
    // edge 0 → 1 in one component; start=2 can reach nothing
    let from = vec![0];
    let to = vec![1];
    let result = graph_bfs::bfs(3, &from, &to, 2, 1);
    let b = match result { Some(_) => false, None => true };
    assert!(b);
}

#[test]
fn test_start_middle() {
    // 0 → 1 → 2; start at 1
    let from = vec![0, 1];
    let to = vec![1, 2];
    let result = graph_bfs::bfs(3, &from, &to, 1, 2);
    let d = match result { Some(x) => x, None => 3 };
    assert_eq!(d, 1);
}

#[test]
fn test_direct_edge() {
    let from = vec![0];
    let to = vec![1];
    let result = graph_bfs::bfs(2, &from, &to, 0, 1);
    let d = match result { Some(x) => x, None => 2 };
    assert_eq!(d, 1);
}

#[test]
fn test_shortest_direct() {
    // 0 → 1 → 2 → 3 (3 edges) and 0 → 3 (1 edge) → shortest is 1
    let from = vec![0, 1, 2, 0];
    let to = vec![1, 2, 3, 3];
    let result = graph_bfs::bfs(4, &from, &to, 0, 3);
    let d = match result { Some(x) => x, None => 4 };
    assert_eq!(d, 1);
}
