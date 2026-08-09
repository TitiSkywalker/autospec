#[test]
fn test_empty() {
    assert_eq!(vector_mean::mean(&[]), 0.0);
}

#[test]
fn test_single() {
    assert_eq!(vector_mean::mean(&[42.0]), 42.0);
}

#[test]
fn test_positive() {
    assert!((vector_mean::mean(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 1e-9);
}

#[test]
fn test_negative() {
    assert!((vector_mean::mean(&[-1.0, -2.0, -3.0]) - (-2.0)).abs() < 1e-9);
}

#[test]
fn test_mixed() {
    assert!((vector_mean::mean(&[-1.0, 0.0, 1.0]) - 0.0).abs() < 1e-9);
}
