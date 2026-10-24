#[test]
fn test_single_edge() {
    // 0 → 1 with capacity 5
    let cap = vec![0, 5, 0, 0];
    let (value, flow) = graph_dinic::dinic(2, &cap, 0, 1);
    assert_eq!(value, 5);
    assert_eq!(flow[1], 5);   // flow[0,1]
    assert_eq!(flow[2], -5);  // flow[1,0]
}

#[test]
fn test_simple() {
    // s=0, a=1, b=2, t=3
    // s→a=3, s→b=2, a→b=1, a→t=2, b→t=3  →  max flow 5
    let cap = vec![
        0, 3, 2, 0,
        0, 0, 1, 2,
        0, 0, 0, 3,
        0, 0, 0, 0,
    ];
    let (value, _flow) = graph_dinic::dinic(4, &cap, 0, 3);
    assert_eq!(value, 5);
}

#[test]
fn test_disconnected() {
    // only edge 0 → 1; sink 2 is unreachable
    let cap = vec![0, 3, 0, 0, 0, 0, 0, 0, 0];
    let (value, _flow) = graph_dinic::dinic(3, &cap, 0, 2);
    assert_eq!(value, 0);
}

#[test]
fn test_diamond() {
    // s=0, a=1, b=2, t=3
    // s→a=10, s→b=10, a→b=1, a→t=10, b→t=10  →  max flow 20
    let cap = vec![
        0, 10, 10, 0,
        0,  0,  1, 10,
        0,  0,  0, 10,
        0,  0,  0,  0,
    ];
    let (value, _flow) = graph_dinic::dinic(4, &cap, 0, 3);
    assert_eq!(value, 20);
}

#[test]
fn test_source_equals_sink() {
    let cap = vec![0, 1, 1, 0, 0, 1, 0, 0, 0];
    let (value, _flow) = graph_dinic::dinic(3, &cap, 1, 1);
    assert_eq!(value, 0);
}

#[test]
fn test_bottleneck() {
    // s=0, a=1, b=2, c=3, t=4
    // s→a=10, s→b=10, a→c=10, b→c=10, c→t=5  →  max flow 5 (c→t is the bottleneck)
    let cap = vec![
        0, 10, 10, 0, 0,
        0,  0,  0, 10, 0,
        0,  0,  0, 10, 0,
        0,  0,  0,  0, 5,
        0,  0,  0,  0, 0,
    ];
    let (value, _flow) = graph_dinic::dinic(5, &cap, 0, 4);
    assert_eq!(value, 5);
}

#[test]
fn test_multiple_phases() {
    // s=0, a=1, b=2, t=3
    // s→t=1 (length 1) plus s→a=1, a→b=1, b→t=1 (length 3)
    // → max flow 2, and Dinic needs a second BFS phase after the direct edge
    // s→t is saturated (levels go 1 → 3).
    let cap = vec![
        0, 1, 0, 1,
        0, 0, 1, 0,
        0, 0, 0, 1,
        0, 0, 0, 0,
    ];
    let (value, _flow) = graph_dinic::dinic(4, &cap, 0, 3);
    assert_eq!(value, 2);
}
