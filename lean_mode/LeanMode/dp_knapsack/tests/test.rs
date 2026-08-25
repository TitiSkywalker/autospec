// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_small_example() {
    // weights: [2, 3, 4, 5], values: [3, 4, 5, 6], capacity: 5
    // Optimal: items with weights 2 and 3 (values 3+4=7)
    let weights = vec![2, 3, 4, 5];
    let values = vec![3, 4, 5, 6];
    let result = dp_knapsack::knapsack(&weights, &values, 5);
    assert_eq!(result, 7);
}

#[test]
fn test_all_items_fit() {
    // All items fit — should take everything
    let weights = vec![1, 2, 3];
    let values = vec![10, 20, 30];
    let result = dp_knapsack::knapsack(&weights, &values, 10);
    assert_eq!(result, 60);
}

#[test]
fn test_none_fit() {
    // No single item fits
    let weights = vec![10, 20, 30];
    let values = vec![100, 200, 300];
    let result = dp_knapsack::knapsack(&weights, &values, 5);
    assert_eq!(result, 0);
}

#[test]
fn test_greedy_fails() {
    // Greedy (by value/weight ratio) would pick item 0 (ratio 15/10=1.5)
    // but the optimal solution is items 1+2 (value 7+9=16, weight 6+4=10)
    let weights = vec![10, 6, 4];
    let values = vec![15, 7, 9];
    let result = dp_knapsack::knapsack(&weights, &values, 10);
    assert_eq!(result, 16);
}

#[test]
fn test_greedy_fails_2() {
    // Greedy would pick item 0 (value 15, weight 10) and stop,
    // but items 1+2 give value 16 with weight 3
    let weights = vec![10, 1, 2];
    let values = vec![15, 9, 7];
    let result = dp_knapsack::knapsack(&weights, &values, 10);
    assert_eq!(result, 16);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn test_empty_input() {
    let weights: Vec<usize> = vec![];
    let values: Vec<usize> = vec![];
    let result = dp_knapsack::knapsack(&weights, &values, 10);
    assert_eq!(result, 0);
}

#[test]
fn test_zero_capacity() {
    let weights = vec![1, 2, 3];
    let values = vec![10, 20, 30];
    let result = dp_knapsack::knapsack(&weights, &values, 0);
    assert_eq!(result, 0);
}

#[test]
fn test_single_item_fits() {
    let weights = vec![5];
    let values = vec![50];
    let result = dp_knapsack::knapsack(&weights, &values, 10);
    assert_eq!(result, 50);
}

#[test]
fn test_single_item_too_heavy() {
    let weights = vec![15];
    let values = vec![100];
    let result = dp_knapsack::knapsack(&weights, &values, 10);
    assert_eq!(result, 0);
}

#[test]
fn test_exact_fit() {
    // Items exactly fill the capacity
    let weights = vec![2, 3, 5];
    let values = vec![10, 20, 30];
    let result = dp_knapsack::knapsack(&weights, &values, 10);
    assert_eq!(result, 60);
}

#[test]
fn test_larger_dataset() {
    let weights = vec![23, 31, 29, 44, 53, 38, 63, 85, 89, 82];
    let values = vec![92, 57, 49, 68, 60, 43, 67, 84, 87, 72];
    let result = dp_knapsack::knapsack(&weights, &values, 165);
    assert_eq!(result, 309);
}

// ── Property tests ──────────────────────────────────────────────────────────

#[test]
fn test_monotonic_capacity() {
    let weights = vec![1, 2, 3, 4];
    let values = vec![10, 20, 30, 40];
    let r1 = dp_knapsack::knapsack(&weights, &values, 5);
    let r2 = dp_knapsack::knapsack(&weights, &values, 10);
    assert!(r2 >= r1);
}

#[test]
fn test_bounded_by_total_value() {
    let weights = vec![1, 2, 3, 4];
    let values = vec![10, 20, 30, 40];
    let total = 100; // 10+20+30+40
    let result = dp_knapsack::knapsack(&weights, &values, 100);
    assert_eq!(result, total);
}