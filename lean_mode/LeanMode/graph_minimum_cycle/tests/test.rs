#[test]
fn test_triangle() {
    // 0-1-2-0, unit weights: min cycle = 3.
    let w = vec![
        0, 1, 1,
        1, 0, 1,
        1, 1, 0,
    ];
    let r = minimum_cycle::minimum_cycle(3, &w);
    let ok = match r { Ok(v) => v, Err(()) => -1 };
    assert_eq!(ok, 3);
}

#[test]
fn test_weighted_triangle() {
    // Edges 0-1=2, 1-2=3, 0-2=4: min cycle = 9.
    let w = vec![
        0, 2, 4,
        2, 0, 3,
        4, 3, 0,
    ];
    let r = minimum_cycle::minimum_cycle(3, &w);
    let ok = match r { Ok(v) => v, Err(()) => -1 };
    assert_eq!(ok, 9);
}

#[test]
fn test_cycle4() {
    // 4-cycle 0-1-2-3-0, unit weights: min cycle = 4.
    let w = vec![
        0, 1, 1_000_000_000_000_000_000i64, 1,
        1, 0, 1, 1_000_000_000_000_000_000i64,
        1_000_000_000_000_000_000i64, 1, 0, 1,
        1, 1_000_000_000_000_000_000i64, 1, 0,
    ];
    let r = minimum_cycle::minimum_cycle(4, &w);
    let ok = match r { Ok(v) => v, Err(()) => -1 };
    assert_eq!(ok, 4);
}

#[test]
fn test_no_cycle() {
    // A single edge 0-1: no cycle.
    let w = vec![
        0, 1,
        1, 0,
    ];
    let r = minimum_cycle::minimum_cycle(2, &w);
    let is_err = match r { Ok(_) => false, Err(()) => true };
    assert!(is_err);
}

#[test]
fn test_cycle_with_diagonal() {
    // 4-cycle 0-1-2-3-0 plus chord 0-2: two triangles of weight 3, min = 3.
    let w = vec![
        0, 1, 1, 1,
        1, 0, 1, 1_000_000_000_000_000_000i64,
        1, 1, 0, 1,
        1, 1_000_000_000_000_000_000i64, 1, 0,
    ];
    let r = minimum_cycle::minimum_cycle(4, &w);
    let ok = match r { Ok(v) => v, Err(()) => -1 };
    assert_eq!(ok, 3);
}
