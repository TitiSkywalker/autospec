#[test]
fn test_single_edge() {
    // 0 → 1, capacity 5, cost 3  →  value 5, cost 15
    let cap = vec![0, 5, 0, 0];
    let cost = vec![0, 3, 0, 0];
    let result = graph_min_cost_flow::min_cost_flow(2, &cap, &cost, 0, 1);
    let (value, total_cost, _flow) = match result {
        Ok(t) => t,
        Err(_) => panic!(),
    };
    assert_eq!(value, 5);
    assert_eq!(total_cost, 15);
}

#[test]
fn test_prefer_cheap_path() {
    // s=0, a=1, t=2
    // s→a cap 10 cost 1, a→t cap 10 cost 1, s→t cap 5 cost 100
    // max flow 15: 10 via s→a→t (cost 2 each), 5 via s→t (cost 100 each)
    let cap = vec![
        0, 10, 5,
        0, 0, 10,
        0, 0, 0,
    ];
    let cost = vec![
        0, 1, 100,
        0, 0, 1,
        0, 0, 0,
    ];
    let result = graph_min_cost_flow::min_cost_flow(3, &cap, &cost, 0, 2);
    let (value, total_cost, _flow) = match result {
        Ok(t) => t,
        Err(_) => panic!(),
    };
    assert_eq!(value, 15);
    assert_eq!(total_cost, 520); // 10 * 2 + 5 * 100
}

#[test]
fn test_reroute() {
    // s=0, a=1, b=2, t=3
    // s→a (1, 1), s→b (1, 4), a→b (1, 1), a→t (1, 4), b→t (1, 1)
    // max flow 2, min cost 10.  The greedy first path s→a→b→t (cost 3) must be
    // re-routed: the second augmenting path s→b→a→t cancels a→b via a reverse
    // edge, yielding the optimal {s→a→t, s→b→t}.
    let cap = vec![
        0, 1, 1, 0,
        0, 0, 1, 1,
        0, 0, 0, 1,
        0, 0, 0, 0,
    ];
    let cost = vec![
        0, 1, 4, 0,
        0, 0, 1, 4,
        0, 0, 0, 1,
        0, 0, 0, 0,
    ];
    let result = graph_min_cost_flow::min_cost_flow(4, &cap, &cost, 0, 3);
    let (value, total_cost, flow) = match result {
        Ok(t) => t,
        Err(_) => panic!(),
    };
    assert_eq!(value, 2);
    assert_eq!(total_cost, 10);
    // Optimal routing: s→a→t and s→b→t, so the a→b edge stays unused.
    assert_eq!(flow[6], 0); // flow[a, b] = row 1 * 4 + col 2
}

#[test]
fn test_negative_cost_no_cycle() {
    // s=0, a=1, t=2
    // s→a cap 2 cost 5, a→t cap 2 cost 5, s→t cap 1 cost -2 (no cycle)
    // max flow 3: 1 unit via s→t (-2), 2 units via s→a→t (10 each)  →  cost 18
    let cap = vec![
        0, 2, 1,
        0, 0, 2,
        0, 0, 0,
    ];
    let cost = vec![
        0, 5, -2,
        0, 0, 5,
        0, 0, 0,
    ];
    let result = graph_min_cost_flow::min_cost_flow(3, &cap, &cost, 0, 2);
    let (value, total_cost, _flow) = match result {
        Ok(t) => t,
        Err(_) => panic!(),
    };
    assert_eq!(value, 3);
    assert_eq!(total_cost, 18); // 1 * -2 + 2 * 10
}

#[test]
fn test_negative_cycle() {
    // 0 → 1 (cost 3), 1 → 0 (cost -4): negative cycle of total cost -1
    let cap = vec![0, 1, 1, 0];
    let cost = vec![0, 3, -4, 0];
    let result = graph_min_cost_flow::min_cost_flow(2, &cap, &cost, 0, 1);
    assert!(!result.is_ok());
}

#[test]
fn test_disconnected() {
    // only edge 0 → 1; sink 2 is unreachable  →  value 0, cost 0
    let cap = vec![
        0, 3, 0,
        0, 0, 0,
        0, 0, 0,
    ];
    let cost = vec![
        0, 1, 0,
        0, 0, 0,
        0, 0, 0,
    ];
    let result = graph_min_cost_flow::min_cost_flow(3, &cap, &cost, 0, 2);
    let (value, total_cost, _flow) = match result {
        Ok(t) => t,
        Err(_) => panic!(),
    };
    assert_eq!(value, 0);
    assert_eq!(total_cost, 0);
}

#[test]
fn test_source_equals_sink() {
    let cap = vec![0, 1, 1, 0, 0, 1, 0, 0, 0];
    let cost = vec![0, 1, 1, 0, 0, 1, 0, 0, 0];
    let result = graph_min_cost_flow::min_cost_flow(3, &cap, &cost, 1, 1);
    let (value, total_cost, _flow) = match result {
        Ok(t) => t,
        Err(_) => panic!(),
    };
    assert_eq!(value, 0);
    assert_eq!(total_cost, 0);
}
