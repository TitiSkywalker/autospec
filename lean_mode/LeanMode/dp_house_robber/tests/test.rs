// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_example_1() {
    let nums = vec![1, 2, 3, 1];
    assert_eq!(dp_house_robber::rob(&nums), 4);
}

#[test]
fn test_example_2() {
    let nums = vec![2, 7, 9, 3, 1];
    assert_eq!(dp_house_robber::rob(&nums), 12);
}

#[test]
fn test_single_house() {
    let nums = vec![5];
    assert_eq!(dp_house_robber::rob(&nums), 5);
}

#[test]
fn test_two_houses() {
    let nums = vec![1, 2];
    assert_eq!(dp_house_robber::rob(&nums), 2);
}

#[test]
fn test_two_houses_pick_first() {
    let nums = vec![100, 1];
    assert_eq!(dp_house_robber::rob(&nums), 100);
}

#[test]
fn test_skip_one_in_middle() {
    let nums = vec![2, 1, 1, 2];
    assert_eq!(dp_house_robber::rob(&nums), 4);
}

#[test]
fn test_alternating() {
    let nums = vec![100, 1, 1, 100];
    assert_eq!(dp_house_robber::rob(&nums), 200);
}

#[test]
fn test_longer_sequence() {
    // 10 + 10 + 10 = 30 (indices 0, 3, 5)
    let nums = vec![10, 1, 1, 10, 1, 10];
    assert_eq!(dp_house_robber::rob(&nums), 30);
}

#[test]
fn test_all_zeros() {
    let nums = vec![0, 0, 0, 0];
    assert_eq!(dp_house_robber::rob(&nums), 0);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn test_empty() {
    let nums: Vec<usize> = vec![];
    assert_eq!(dp_house_robber::rob(&nums), 0);
}
