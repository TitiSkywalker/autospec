use array_floyd_cycle::floyd_find_cycle;

/// Test: self-loop at the start node
#[test]
fn test_self_loop() {
    let f = vec![0usize];
    let result = floyd_find_cycle(&f, 0);
    assert_eq!(result, 0);
}

/// Test: pure cycle 0 → 1 → 2 → 3 → 0
#[test]
fn test_pure_cycle() {
    let f = vec![1, 2, 3, 0];
    let result = floyd_find_cycle(&f, 0);
    assert_eq!(result, 0);
}

/// Test: tail then cycle (0 → 1 → 2 → 3 → 3), entry is 3
#[test]
fn test_tail_then_self_loop() {
    let f = vec![1, 2, 3, 3];
    let result = floyd_find_cycle(&f, 0);
    assert_eq!(result, 3);
}

/// Test: longer tail then 4-cycle (0 → 1 → 2 → 3 → 4 → 5 → 6 → 3), entry is 3
#[test]
fn test_tail_then_cycle() {
    let f = vec![1, 2, 3, 4, 5, 6, 3];
    let result = floyd_find_cycle(&f, 0);
    assert_eq!(result, 3);
}

/// Test: start inside the cycle (start at 4 in 3 → 4 → 5 → 6 → 3), entry is 4
#[test]
fn test_start_inside_cycle() {
    let f = vec![1, 2, 3, 4, 5, 6, 3];
    let result = floyd_find_cycle(&f, 4);
    assert_eq!(result, 4);
}

/// Test: two-element cycle
#[test]
fn test_two_cycle() {
    let f = vec![1, 2, 0, 4, 3];
    // 0 -> 1 -> 2 -> 0 (cycle 0,1,2), 3 <-> 4 (cycle 3,4)
    let result = floyd_find_cycle(&f, 0);
    assert_eq!(result, 0);
    let result2 = floyd_find_cycle(&f, 3);
    assert_eq!(result2, 3);
}

/// Test: longer tail into the entry point
#[test]
fn test_long_tail() {
    let f = vec![1, 2, 3, 4, 5, 5];
    // 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 5 (entry 5)
    let result = floyd_find_cycle(&f, 0);
    assert_eq!(result, 5);
}
