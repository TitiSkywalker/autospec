#[test]
fn test_empty() {
    assert_eq!(array_max::max(&[]), 0.0);
}

#[test]
fn test_single() {
    assert_eq!(array_max::max(&[42.0]), 42.0);
}

#[test]
fn test_positive() {
    assert!((array_max::max(&[1.0, 5.0, 3.0, 4.0, 2.0]) - 5.0).abs() < 1e-9);
}

#[test]
fn test_negative() {
    assert!((array_max::max(&[-5.0, -1.0, -3.0]) - (-1.0)).abs() < 1e-9);
}

#[test]
fn test_mixed_signs() {
    assert!((array_max::max(&[-2.0, 0.0, 3.0, -1.0]) - 3.0).abs() < 1e-9);
}
