#[test]
fn test_simple_path() {
    // 0→1 (1), 1→2 (2), 0→2 (10)  ⇒  0→2 via 1 is 3
    let weight = vec![
        Some(0), Some(1), Some(10),
        None, Some(0), Some(2),
        None, None, Some(0),
    ];
    let result = graph_floyd_warshall::floyd_warshall(3, &weight);
    let dist = match result {
        Some(d) => d,
        None => panic!(),
    };
    match dist[0] { Some(x) => assert_eq!(x, 0), None => panic!() };
    match dist[1] { Some(x) => assert_eq!(x, 1), None => panic!() };
    match dist[2] { Some(x) => assert_eq!(x, 3), None => panic!() };
    match dist[3] { Some(_) => panic!(), None => {} };
    match dist[4] { Some(x) => assert_eq!(x, 0), None => panic!() };
    match dist[5] { Some(x) => assert_eq!(x, 2), None => panic!() };
    match dist[6] { Some(_) => panic!(), None => {} };
    match dist[7] { Some(_) => panic!(), None => {} };
    match dist[8] { Some(x) => assert_eq!(x, 0), None => panic!() };
}

#[test]
fn test_negative_weight() {
    // 0→1 (-1), 1→2 (2), 0→2 (1)  ⇒  0→2 = min(1, -1+2) = 1
    let weight = vec![
        Some(0), Some(-1), Some(1),
        None, Some(0), Some(2),
        None, None, Some(0),
    ];
    let result = graph_floyd_warshall::floyd_warshall(3, &weight);
    let dist = match result {
        Some(d) => d,
        None => panic!(),
    };
    match dist[0] { Some(x) => assert_eq!(x, 0), None => panic!() };
    match dist[1] { Some(x) => assert_eq!(x, -1), None => panic!() };
    match dist[2] { Some(x) => assert_eq!(x, 1), None => panic!() };
    match dist[3] { Some(_) => panic!(), None => {} };
    match dist[4] { Some(x) => assert_eq!(x, 0), None => panic!() };
    match dist[5] { Some(x) => assert_eq!(x, 2), None => panic!() };
    match dist[6] { Some(_) => panic!(), None => {} };
    match dist[7] { Some(_) => panic!(), None => {} };
    match dist[8] { Some(x) => assert_eq!(x, 0), None => panic!() };
}

#[test]
fn test_negative_cycle() {
    // 0→1 (-1), 1→0 (-2)  ⇒  negative cycle
    let weight = vec![Some(0), Some(-1), Some(-2), Some(0)];
    let result = graph_floyd_warshall::floyd_warshall(2, &weight);
    match result {
        Some(_) => panic!(),
        None => {}
    }
}

#[test]
fn test_single_node() {
    let weight = vec![Some(0)];
    let result = graph_floyd_warshall::floyd_warshall(1, &weight);
    let dist = match result {
        Some(d) => d,
        None => panic!(),
    };
    match dist[0] { Some(x) => assert_eq!(x, 0), None => panic!() };
}

#[test]
fn test_no_edges() {
    let weight = vec![Some(0), None, None, Some(0)];
    let result = graph_floyd_warshall::floyd_warshall(2, &weight);
    let dist = match result {
        Some(d) => d,
        None => panic!(),
    };
    match dist[0] { Some(x) => assert_eq!(x, 0), None => panic!() };
    match dist[1] { Some(_) => panic!(), None => {} };
    match dist[2] { Some(_) => panic!(), None => {} };
    match dist[3] { Some(x) => assert_eq!(x, 0), None => panic!() };
}

#[test]
fn test_transitive() {
    // 0→1 (2), 1→2 (3)  ⇒  0→2 = 5
    let weight = vec![
        Some(0), Some(2), None,
        None, Some(0), Some(3),
        None, None, Some(0),
    ];
    let result = graph_floyd_warshall::floyd_warshall(3, &weight);
    let dist = match result {
        Some(d) => d,
        None => panic!(),
    };
    match dist[0] { Some(x) => assert_eq!(x, 0), None => panic!() };
    match dist[1] { Some(x) => assert_eq!(x, 2), None => panic!() };
    match dist[2] { Some(x) => assert_eq!(x, 5), None => panic!() };
    match dist[3] { Some(_) => panic!(), None => {} };
    match dist[4] { Some(x) => assert_eq!(x, 0), None => panic!() };
    match dist[5] { Some(x) => assert_eq!(x, 3), None => panic!() };
    match dist[6] { Some(_) => panic!(), None => {} };
    match dist[7] { Some(_) => panic!(), None => {} };
    match dist[8] { Some(x) => assert_eq!(x, 0), None => panic!() };
}
