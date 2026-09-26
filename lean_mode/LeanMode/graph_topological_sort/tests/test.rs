#[test]
fn test_chain() {
    // 0 → 1 → 2 → 3
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 3];
    let result = graph_topological_sort::topological_sort(4, &from, &to);
    let order = match result {
        Some(o) => o,
        None => panic!(),
    };
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], 0);
    assert_eq!(order[1], 1);
    assert_eq!(order[2], 2);
    assert_eq!(order[3], 3);
}

#[test]
fn test_diamond() {
    // 0 → 1, 0 → 2, 1 → 3, 2 → 3
    let from = vec![0, 0, 1, 2];
    let to = vec![1, 2, 3, 3];
    let result = graph_topological_sort::topological_sort(4, &from, &to);
    let order = match result {
        Some(o) => o,
        None => panic!(),
    };
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], 0);
    assert_eq!(order[1], 1);
    assert_eq!(order[2], 2);
    assert_eq!(order[3], 3);
}

#[test]
fn test_cycle() {
    // 0 → 1 → 2 → 0
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 0];
    let result = graph_topological_sort::topological_sort(3, &from, &to);
    match result {
        Some(_) => panic!(),
        None => {}
    }
}

#[test]
fn test_single_node() {
    let result = graph_topological_sort::topological_sort(1, &[], &[]);
    let order = match result {
        Some(o) => o,
        None => panic!(),
    };
    assert_eq!(order.len(), 1);
    assert_eq!(order[0], 0);
}

#[test]
fn test_empty_graph() {
    let result = graph_topological_sort::topological_sort(3, &[], &[]);
    let order = match result {
        Some(o) => o,
        None => panic!(),
    };
    assert_eq!(order.len(), 3);
    assert_eq!(order[0], 0);
    assert_eq!(order[1], 1);
    assert_eq!(order[2], 2);
}

#[test]
fn test_disconnected() {
    // only edge 0 → 1; nodes 2 and 3 isolated
    let from = vec![0];
    let to = vec![1];
    let result = graph_topological_sort::topological_sort(4, &from, &to);
    let order = match result {
        Some(o) => o,
        None => panic!(),
    };
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], 0);
    assert_eq!(order[1], 2);
    assert_eq!(order[2], 3);
    assert_eq!(order[3], 1);
}

#[test]
fn test_cycle_with_dangling() {
    // cycle 0 → 1 → 2 → 0, plus isolated node 3
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 0];
    let result = graph_topological_sort::topological_sort(4, &from, &to);
    match result {
        Some(_) => panic!(),
        None => {}
    }
}

#[test]
fn test_self_loop() {
    // 0 → 0
    let from = vec![0];
    let to = vec![0];
    let result = graph_topological_sort::topological_sort(1, &from, &to);
    match result {
        Some(_) => panic!(),
        None => {}
    }
}
