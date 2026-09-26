#[test]
fn test_even_cycle() {
    // square 0-1-2-3-0
    let from = vec![0, 1, 2, 3];
    let to = vec![1, 2, 3, 0];
    let result = graph_bipartite_check::is_bipartite(4, &from, &to);
    let colors = match result {
        Some(c) => c,
        None => panic!(),
    };
    assert_eq!(colors.len(), 4);
    assert_eq!(colors[0], 0);
    assert_eq!(colors[1], 1);
    assert_eq!(colors[2], 0);
    assert_eq!(colors[3], 1);
}

#[test]
fn test_triangle_not_bipartite() {
    // triangle 0-1-2-0 (odd cycle)
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 0];
    let result = graph_bipartite_check::is_bipartite(3, &from, &to);
    match result {
        Some(_) => panic!(),
        None => {}
    }
}

#[test]
fn test_single_node() {
    let result = graph_bipartite_check::is_bipartite(1, &[], &[]);
    let colors = match result {
        Some(c) => c,
        None => panic!(),
    };
    assert_eq!(colors.len(), 1);
    assert_eq!(colors[0], 0);
}

#[test]
fn test_empty_graph() {
    let result = graph_bipartite_check::is_bipartite(3, &[], &[]);
    let colors = match result {
        Some(c) => c,
        None => panic!(),
    };
    assert_eq!(colors.len(), 3);
    assert_eq!(colors[0], 0);
    assert_eq!(colors[1], 0);
    assert_eq!(colors[2], 0);
}

#[test]
fn test_two_components() {
    // edges {0,1} and {2,3}
    let from = vec![0, 2];
    let to = vec![1, 3];
    let result = graph_bipartite_check::is_bipartite(4, &from, &to);
    let colors = match result {
        Some(c) => c,
        None => panic!(),
    };
    assert_eq!(colors.len(), 4);
    assert_eq!(colors[0], 0);
    assert_eq!(colors[1], 1);
    assert_eq!(colors[2], 0);
    assert_eq!(colors[3], 1);
}

#[test]
fn test_star() {
    // center 0 connected to 1, 2, 3
    let from = vec![0, 0, 0];
    let to = vec![1, 2, 3];
    let result = graph_bipartite_check::is_bipartite(4, &from, &to);
    let colors = match result {
        Some(c) => c,
        None => panic!(),
    };
    assert_eq!(colors.len(), 4);
    assert_eq!(colors[0], 0);
    assert_eq!(colors[1], 1);
    assert_eq!(colors[2], 1);
    assert_eq!(colors[3], 1);
}

#[test]
fn test_self_loop_not_bipartite() {
    // self-loop {0,0}
    let from = vec![0];
    let to = vec![0];
    let result = graph_bipartite_check::is_bipartite(1, &from, &to);
    match result {
        Some(_) => panic!(),
        None => {}
    }
}

#[test]
fn test_odd_cycle_not_bipartite() {
    // 5-cycle 0-1-2-3-4-0 (odd cycle)
    let from = vec![0, 1, 2, 3, 4];
    let to = vec![1, 2, 3, 4, 0];
    let result = graph_bipartite_check::is_bipartite(5, &from, &to);
    match result {
        Some(_) => panic!(),
        None => {}
    }
}
