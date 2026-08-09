use vector_norm::vector_norm;

/// Test: basic 2D vector
#[test]
fn test_basic_2d() {
    let v = vec![3.0, 4.0];
    let n = vector_norm(&v, 2);
    assert!((n - 5.0).abs() < 1e-10);
}

/// Test: basic 3D vector
#[test]
fn test_basic_3d() {
    let v = vec![1.0, 2.0, 2.0];
    let n = vector_norm(&v, 3);
    assert!((n - 3.0).abs() < 1e-10);
}

/// Test: zero vector
#[test]
fn test_zero() {
    let v = vec![0.0, 0.0, 0.0];
    let n = vector_norm(&v, 3);
    assert!(n.abs() < 1e-10);
}

/// Test: single element
#[test]
fn test_single() {
    let v = vec![5.0];
    let n = vector_norm(&v, 1);
    assert!((n - 5.0).abs() < 1e-10);
}

/// Test: empty vector
#[test]
fn test_empty() {
    let v: Vec<f64> = vec![];
    let n = vector_norm(&v, 0);
    assert!(n.abs() < 1e-10);
}

/// Test: negative values
#[test]
fn test_negatives() {
    let v = vec![-1.0, -2.0, -3.0];
    let n = vector_norm(&v, 3);
    let expected = (14.0_f64).sqrt();
    assert!((n - expected).abs() < 1e-10);
}

/// Test: unit vector
#[test]
fn test_unit() {
    let v = vec![0.0, 1.0, 0.0];
    let n = vector_norm(&v, 3);
    assert!((n - 1.0).abs() < 1e-10);
}
