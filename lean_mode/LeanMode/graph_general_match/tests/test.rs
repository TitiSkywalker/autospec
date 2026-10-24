#[test]
fn test_single_edge() {
    let adj = vec![0, 1, 1, 0];
    let (size, matched) = graph_general_match::blossom(2, &adj);
    assert_eq!(size, 1);
    assert_eq!(matched[1], 1);
    assert_eq!(matched[2], 1);
}

#[test]
fn test_triangle() {
    // A triangle (odd cycle): max matching is 1 (only one edge can be chosen).
    let adj = vec![
        0, 1, 1,
        1, 0, 1,
        1, 1, 0,
    ];
    let (size, _matched) = graph_general_match::blossom(3, &adj);
    assert_eq!(size, 1);
}

#[test]
fn test_pentagon() {
    // A 5-cycle (odd): max matching is 2.
    let adj = vec![
        0, 1, 0, 0, 1,
        1, 0, 1, 0, 0,
        0, 1, 0, 1, 0,
        0, 0, 1, 0, 1,
        1, 0, 0, 1, 0,
    ];
    let (size, _matched) = graph_general_match::blossom(5, &adj);
    assert_eq!(size, 2);
}

#[test]
fn test_path4() {
    // 0-1-2-3: max matching 2 (0-1 and 2-3).
    let adj = vec![
        0, 1, 0, 0,
        1, 0, 1, 0,
        0, 1, 0, 1,
        0, 0, 1, 0,
    ];
    let (size, _matched) = graph_general_match::blossom(4, &adj);
    assert_eq!(size, 2);
}

#[test]
fn test_no_edges() {
    let adj = vec![0, 0, 0, 0, 0, 0, 0, 0, 0];
    let (size, _matched) = graph_general_match::blossom(3, &adj);
    assert_eq!(size, 0);
}
