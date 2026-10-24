#[test]
fn test_single_edge() {
    let adj = vec![1]; // u0-v0
    let (size, matched) = graph_hopcroft_karp::hopcroft_karp(1, 1, &adj);
    assert_eq!(size, 1);
    assert_eq!(matched[0], 1);
}

#[test]
fn test_augmenting_path() {
    // left u0,u1; right v0,v1.  edges: u0-v0, u0-v1, u1-v0.
    // A greedy u0-v0 first would block u1; an augmenting path u1-v0-u0-v1
    // re-routes to reach size 2.
    let adj = vec![
        1, 1,
        1, 0,
    ];
    let (size, matched) = graph_hopcroft_karp::hopcroft_karp(2, 2, &adj);
    assert_eq!(size, 2);
    assert_eq!(matched[0] + matched[1] + matched[2] + matched[3], 2);
    assert_eq!(matched[1], 1); // u0-v1
    assert_eq!(matched[2], 1); // u1-v0
}

#[test]
fn test_disjoint() {
    // u0-v0, u1-v1
    let adj = vec![
        1, 0,
        0, 1,
    ];
    let (size, _matched) = graph_hopcroft_karp::hopcroft_karp(2, 2, &adj);
    assert_eq!(size, 2);
}

#[test]
fn test_no_edges() {
    let adj = vec![
        0, 0,
        0, 0,
    ];
    let (size, _matched) = graph_hopcroft_karp::hopcroft_karp(2, 2, &adj);
    assert_eq!(size, 0);
}

#[test]
fn test_star() {
    // all three left vertices adjacent only to v0 → size 1
    let adj = vec![1, 0, 1, 0, 1, 0];
    let (size, _matched) = graph_hopcroft_karp::hopcroft_karp(3, 2, &adj);
    assert_eq!(size, 1);
}

#[test]
fn test_chain() {
    // left u0,u1,u2; right v0,v1,v2.  edges: u0-v0, u1-v0, u1-v1, u2-v1.
    // max matching = 2 (e.g. u0-v0, u1-v1)
    let adj = vec![
        1, 0, 0,
        1, 1, 0,
        0, 1, 0,
    ];
    let (size, _matched) = graph_hopcroft_karp::hopcroft_karp(3, 3, &adj);
    assert_eq!(size, 2);
}

#[test]
fn test_larger() {
    // 3x3: u0-v0,u0-v1; u1-v1,u1-v2; u2-v0,u2-v2  → max matching 3
    let adj = vec![
        1, 1, 0,
        0, 1, 1,
        1, 0, 1,
    ];
    let (size, _matched) = graph_hopcroft_karp::hopcroft_karp(3, 3, &adj);
    assert_eq!(size, 3);
}
