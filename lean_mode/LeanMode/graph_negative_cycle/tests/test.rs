#[test]
fn test_no_negative_cycle() {
    // 0 -> 1 (cost 2), 1 -> 2 (cost 3), 0 -> 2 (cost 10): no negative cycle.
    let cap = vec![
        0, 1, 1,
        0, 0, 1,
        0, 0, 0,
    ];
    let cost = vec![
        0, 2, 10,
        0, 0, 3,
        0, 0, 0,
    ];
    let (has_neg, _h) = negative_cycle::negative_cycle(3, &cap, &cost);
    assert!(!has_neg);
}

#[test]
fn test_negative_cycle() {
    // 0 -> 1 (1), 1 -> 0 (-3): a negative 2-cycle of cost -2.
    let cap = vec![
        0, 1,
        1, 0,
    ];
    let cost = vec![
        0, 1,
        -3, 0,
    ];
    let (has_neg, _h) = negative_cycle::negative_cycle(2, &cap, &cost);
    assert!(has_neg);
}

#[test]
fn test_three_cycle() {
    // 0 -> 1 (1), 1 -> 2 (1), 2 -> 0 (-3): a negative 3-cycle.
    let cap = vec![
        0, 1, 0,
        0, 0, 1,
        1, 0, 0,
    ];
    let cost = vec![
        0, 1, 0,
        0, 0, 1,
        -3, 0, 0,
    ];
    let (has_neg, _h) = negative_cycle::negative_cycle(3, &cap, &cost);
    assert!(has_neg);
}

#[test]
fn test_self_loop_negative() {
    // A negative self-loop on vertex 0.
    let cap = vec![1];
    let cost = vec![-1];
    let (has_neg, _h) = negative_cycle::negative_cycle(1, &cap, &cost);
    assert!(has_neg);
}

#[test]
fn test_no_edges() {
    // No edges at all: no negative cycle.
    let cap = vec![0, 0, 0, 0];
    let cost = vec![0, 0, 0, 0];
    let (has_neg, _h) = negative_cycle::negative_cycle(2, &cap, &cost);
    assert!(!has_neg);
}
