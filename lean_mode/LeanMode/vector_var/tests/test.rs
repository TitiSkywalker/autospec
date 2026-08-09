#[test]
fn test_empty() {
    assert_eq!(vector_var::var(&[]), 0.0);
}

#[test]
fn test_single() {
    assert_eq!(vector_var::var(&[42.0]), 0.0);
}

#[test]
fn test_constant() {
    assert!((vector_var::var(&[5.0, 5.0, 5.0, 5.0]) - 0.0).abs() < 1e-9);
}

#[test]
fn test_two_values() {
    assert!((vector_var::var(&[1.0, 3.0]) - 1.0).abs() < 1e-9);
}

#[test]
fn test_general() {
    let v = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    // Expected population variance:
    // mean = 5.0, deviations: -3, -1, -1, -1, 0, 0, 2, 4
    // squared: 9 + 1 + 1 + 1 + 0 + 0 + 4 + 16 = 32, variance = 32/8 = 4.0
    assert!((vector_var::var(&v) - 4.0).abs() < 1e-9);
}
