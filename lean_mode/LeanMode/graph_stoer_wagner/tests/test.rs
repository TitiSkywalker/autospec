#[test]
fn test_single_edge() {
    // 0 — 1 with weight 5  →  min cut 5
    let weight = vec![0, 5, 5, 0];
    let value = graph_stoer_wagner::stoer_wagner(2, &weight);
    assert_eq!(value, 5);
}

#[test]
fn test_triangle() {
    // 0-1 = 2, 0-2 = 4, 1-2 = 3  →  min cut 5 (isolate vertex 1)
    let weight = vec![
        0, 2, 4,
        2, 0, 3,
        4, 3, 0,
    ];
    let value = graph_stoer_wagner::stoer_wagner(3, &weight);
    assert_eq!(value, 5);
}

#[test]
fn test_cycle4() {
    // 0-1-2-3-0, unit weights  →  min cut 2 (isolate one vertex)
    let weight = vec![
        0, 1, 0, 1,
        1, 0, 1, 0,
        0, 1, 0, 1,
        1, 0, 1, 0,
    ];
    let value = graph_stoer_wagner::stoer_wagner(4, &weight);
    assert_eq!(value, 2);
}

#[test]
fn test_disconnected() {
    // only edge 0 — 1; vertex 2 isolated  →  min cut 0
    let weight = vec![
        0, 7, 0,
        7, 0, 0,
        0, 0, 0,
    ];
    let value = graph_stoer_wagner::stoer_wagner(3, &weight);
    assert_eq!(value, 0);
}

#[test]
fn test_barbell() {
    // two heavy cliques {0,1} and {2,3} joined by a light bridge 1-2
    // 0-1 = 10, 1-2 = 1, 2-3 = 10  →  min cut 1 (the bridge, a balanced cut)
    let weight = vec![
        0, 10, 0, 0,
        10, 0, 1, 0,
        0, 1, 0, 10,
        0, 0, 10, 0,
    ];
    let value = graph_stoer_wagner::stoer_wagner(4, &weight);
    assert_eq!(value, 1);
}

#[test]
fn test_complete_k4() {
    // K4 with unit edges  →  min cut 3 (isolate one vertex)
    let weight = vec![
        0, 1, 1, 1,
        1, 0, 1, 1,
        1, 1, 0, 1,
        1, 1, 1, 0,
    ];
    let value = graph_stoer_wagner::stoer_wagner(4, &weight);
    assert_eq!(value, 3);
}
