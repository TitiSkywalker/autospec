// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_example_1() {
    let nums = vec![2, 3, 1, 1, 4];
    assert_eq!(dp_jump_game::can_jump(&nums), true);
}

#[test]
fn test_example_2() {
    let nums = vec![3, 2, 1, 0, 4];
    assert_eq!(dp_jump_game::can_jump(&nums), false);
}

#[test]
fn test_single_zero() {
    let nums = vec![0];
    assert_eq!(dp_jump_game::can_jump(&nums), true);
}

#[test]
fn test_two_zeros() {
    let nums = vec![0, 0];
    assert_eq!(dp_jump_game::can_jump(&nums), false);
}

#[test]
fn test_jump_two() {
    let nums = vec![2, 0, 0];
    assert_eq!(dp_jump_game::can_jump(&nums), true);
}

#[test]
fn test_stuck_after_one() {
    let nums = vec![1, 0, 1, 0];
    assert_eq!(dp_jump_game::can_jump(&nums), false);
}

#[test]
fn test_cant_move_from_zero() {
    let nums = vec![0, 2, 3];
    assert_eq!(dp_jump_game::can_jump(&nums), false);
}

#[test]
fn test_exact_reach() {
    let nums = vec![1, 1, 1, 0];
    assert_eq!(dp_jump_game::can_jump(&nums), true);
}
