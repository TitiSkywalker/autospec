#[test]
fn test_simple_cycle() {
    // 0 → 1 (low 1, cap 3) and 1 → 0 (low 1, cap 3): feasible (both carry 1..3).
    let low = vec![0, 1, 1, 0];
    let cap = vec![0, 3, 3, 0];
    let result = graph_flow_bound::flow_bound(2, &low, &cap);
    let flow = match result {
        Ok(f) => f,
        Err(()) => panic!(),
    };
    assert!(flow[1] >= 1 && flow[1] <= 3); // flow[0,1]
    assert!(flow[2] >= 1 && flow[2] <= 3); // flow[1,0]
    assert_eq!(flow[0], 0);
    assert_eq!(flow[3], 0);
}

#[test]
fn test_infeasible() {
    // 0 → 1 (low 1, cap 3), no edge back: node 0 must send but has no inflow.
    let low = vec![0, 1, 0, 0];
    let cap = vec![0, 3, 0, 0];
    let result = graph_flow_bound::flow_bound(2, &low, &cap);
    let is_err = match result {
        Ok(_) => false,
        Err(()) => true,
    };
    assert!(is_err);
}

#[test]
fn test_feasible_chain() {
    // 0→1 (low 1, cap 5), 1→2 (low 1, cap 5), 2→0 (low 2, cap 5).
    // A feasible circulation: all edges carry 2.
    let low = vec![
        0, 1, 0,
        0, 0, 1,
        2, 0, 0,
    ];
    let cap = vec![
        0, 5, 0,
        0, 0, 5,
        5, 0, 0,
    ];
    let result = graph_flow_bound::flow_bound(3, &low, &cap);
    let flow = match result {
        Ok(f) => f,
        Err(()) => panic!(),
    };
    // conservation at each vertex
    assert_eq!(flow[0 * 3 + 1], flow[2 * 3 + 0]); // out(0)=in(0)
    assert_eq!(flow[1 * 3 + 2], flow[0 * 3 + 1]); // out(1)=in(1)
    assert_eq!(flow[2 * 3 + 0], flow[1 * 3 + 2]); // out(2)=in(2)
    assert!(flow[0 * 3 + 1] >= 1 && flow[0 * 3 + 1] <= 5);
    assert!(flow[1 * 3 + 2] >= 1 && flow[1 * 3 + 2] <= 5);
    assert!(flow[2 * 3 + 0] >= 2 && flow[2 * 3 + 0] <= 5);
}

#[test]
fn test_zero_lower_bound() {
    // 0 → 1 (low 0, cap 4), 1 → 0 (low 0, cap 4): zero circulation is feasible.
    let low = vec![0, 0, 0, 0];
    let cap = vec![0, 4, 4, 0];
    let result = graph_flow_bound::flow_bound(2, &low, &cap);
    let flow = match result {
        Ok(f) => f,
        Err(()) => panic!(),
    };
    assert!(flow[1] >= 0 && flow[1] <= 4);
    assert!(flow[2] >= 0 && flow[2] <= 4);
}

#[test]
fn test_self_balance() {
    // Single self-contained feasible edge is impossible with a loop, so use a
    // 2-cycle where lower bounds force flow.
    // 0→1 (low 3, cap 3), 1→0 (low 3, cap 3): the unique feasible flow is 3/3.
    let low = vec![0, 3, 3, 0];
    let cap = vec![0, 3, 3, 0];
    let result = graph_flow_bound::flow_bound(2, &low, &cap);
    let flow = match result {
        Ok(f) => f,
        Err(()) => panic!(),
    };
    assert_eq!(flow[1], 3);
    assert_eq!(flow[2], 3);
}
