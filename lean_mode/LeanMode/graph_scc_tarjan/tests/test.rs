#[test]
fn test_cycle() {
    // 0 → 1 → 2 → 0 : one SCC {0,1,2}
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 0];
    let comp = graph_scc_tarjan::scc_tarjan(3, &from, &to);
    assert_eq!(comp[0], comp[1]);
    assert_eq!(comp[1], comp[2]);
    assert_eq!(comp[0], comp[2]);
}

#[test]
fn test_chain() {
    // 0 → 1 → 2 → 3 : every vertex is its own SCC
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 3];
    let comp = graph_scc_tarjan::scc_tarjan(4, &from, &to);
    assert!(comp[0] != comp[1]);
    assert!(comp[1] != comp[2]);
    assert!(comp[2] != comp[3]);
    assert!(comp[0] != comp[3]);
}

#[test]
fn test_two_components() {
    // 0 ⇄ 1 and 2 ⇄ 3 : two SCCs {0,1}, {2,3}
    let from = vec![0, 1, 2, 3];
    let to = vec![1, 0, 3, 2];
    let comp = graph_scc_tarjan::scc_tarjan(4, &from, &to);
    assert_eq!(comp[0], comp[1]);
    assert_eq!(comp[2], comp[3]);
    assert!(comp[0] != comp[2]);
}

#[test]
fn test_sink_component() {
    // 0 → 1 → 2, 2 → 1 : SCCs {0}, {1,2}
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 1];
    let comp = graph_scc_tarjan::scc_tarjan(3, &from, &to);
    assert_eq!(comp[1], comp[2]);
    assert!(comp[0] != comp[1]);
}

#[test]
fn test_dag() {
    // 0 → 1, 0 → 2, 1 → 2 : three SCCs {0}, {1}, {2}
    let from = vec![0, 0, 1];
    let to = vec![1, 2, 2];
    let comp = graph_scc_tarjan::scc_tarjan(3, &from, &to);
    assert!(comp[0] != comp[1]);
    assert!(comp[0] != comp[2]);
    assert!(comp[1] != comp[2]);
}

#[test]
fn test_single_node() {
    let comp = graph_scc_tarjan::scc_tarjan(1, &[], &[]);
    assert_eq!(comp.len(), 1);
}

#[test]
fn test_self_loop() {
    // 0 → 0 : single SCC {0}
    let from = vec![0];
    let to = vec![0];
    let comp = graph_scc_tarjan::scc_tarjan(1, &from, &to);
    assert_eq!(comp.len(), 1);
}

#[test]
fn test_disconnected() {
    // only edge 0 → 1; node 2 isolated : SCCs {0}, {1}, {2}
    let from = vec![0];
    let to = vec![1];
    let comp = graph_scc_tarjan::scc_tarjan(3, &from, &to);
    assert!(comp[0] != comp[1]);
    assert!(comp[0] != comp[2]);
    assert!(comp[1] != comp[2]);
}

#[test]
fn test_cycle_with_tail() {
    // 0 → 1 → 2 → 0 and 2 → 3 : SCCs {0,1,2}, {3}
    let from = vec![0, 1, 2, 2];
    let to = vec![1, 2, 0, 3];
    let comp = graph_scc_tarjan::scc_tarjan(4, &from, &to);
    assert_eq!(comp[0], comp[1]);
    assert_eq!(comp[1], comp[2]);
    assert!(comp[2] != comp[3]);
}
