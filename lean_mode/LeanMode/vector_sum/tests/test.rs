#[test]
fn test_empty() {
    assert_eq!(vector_sum::sum(&[]), 0.0);
}

#[test]
fn test_single() {
    assert_eq!(vector_sum::sum(&[42.0]), 42.0);
}

#[test]
fn test_positive() {
    assert!((vector_sum::sum(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 15.0).abs() < 1e-9);
}

#[test]
fn test_negative() {
    assert!((vector_sum::sum(&[-1.0, 2.0, -3.0, 4.0]) - 2.0).abs() < 1e-9);
}

#[test]
fn test_zero_sum() {
    assert!((vector_sum::sum(&[1.0, -1.0, 2.0, -2.0]) - 0.0).abs() < 1e-9);
}
