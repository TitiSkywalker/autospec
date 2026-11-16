// Translated tests: every `#eval` must yield `Result.ok`.
// Assertions compare the returned fraction `num / den` against a known mean by
// cross-multiplication (all quantities are small).  `INF` = no edge.

#[test]
fn test_unit_triangle() {
    // 3-cycle 0->1->2->0, unit weights, no self-loops: mean = 1, so num == den.
    let w = vec![
        1_000_000_000_000_000_000i64, 1, 1_000_000_000_000_000_000i64,
        1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64, 1,
        1, 1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64,
    ];
    let r = graph_min_mean_cycle::min_mean_cycle(3, &w);
    let (num, den) = match r {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    let lhs = num * 1;
    let rhs = den * 1;
    assert_eq!(lhs, rhs);
}

#[test]
fn test_weighted_triangle() {
    // 3-cycle with weights 2, 3, 4: mean = 3.
    let w = vec![
        1_000_000_000_000_000_000i64, 2, 1_000_000_000_000_000_000i64,
        1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64, 3,
        4, 1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64,
    ];
    let r = graph_min_mean_cycle::min_mean_cycle(3, &w);
    let (num, den) = match r {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    let lhs = num * 1;
    let rhs = den * 3;
    assert_eq!(lhs, rhs);
}

#[test]
fn test_two_cycle() {
    // Directed 2-cycle 0->1 (5), 1->0 (1): mean = (5 + 1) / 2 = 3.
    let w = vec![1_000_000_000_000_000_000i64, 5, 1, 1_000_000_000_000_000_000i64];
    let r = graph_min_mean_cycle::min_mean_cycle(2, &w);
    let (num, den) = match r {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    let lhs = num * 1;
    let rhs = den * 3;
    assert_eq!(lhs, rhs);
}

#[test]
fn test_negative_cycle() {
    // Negative 3-cycle 0->1->2->0 with weights -1, -1, -1: mean = -1.
    let w = vec![
        1_000_000_000_000_000_000i64, -1, 1_000_000_000_000_000_000i64,
        1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64, -1,
        -1, 1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64,
    ];
    let r = graph_min_mean_cycle::min_mean_cycle(3, &w);
    let (num, den) = match r {
        Ok(inner) => inner,
        Err(_) => panic!(),
    };
    let lhs = num * 1;
    let rhs = den * -1;
    assert_eq!(lhs, rhs);
}

#[test]
fn test_no_cycle() {
    // DAG 0->1->2: no cycle, so the result must be an error.
    let w = vec![
        1_000_000_000_000_000_000i64, 1, 1_000_000_000_000_000_000i64,
        1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64, 1,
        1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64, 1_000_000_000_000_000_000i64,
    ];
    let r = graph_min_mean_cycle::min_mean_cycle(3, &w);
    let is_err = match r {
        Ok(_) => false,
        Err(()) => true,
    };
    assert!(is_err);
}
