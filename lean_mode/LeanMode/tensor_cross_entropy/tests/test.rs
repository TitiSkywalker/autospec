use tensor_cross_entropy::cross_entropy;

fn approx(a: f64, b: f64) -> bool {
    let d = if a - b < 0.0 { b - a } else { a - b };
    d < 1e-12
}

#[test]
fn test_cross_entropy_basic() {
    let pred = vec![0.7, 0.2, 0.1, 0.1, 0.8, 0.1];
    let target = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let loss = cross_entropy(&pred, &target, 2, 3);
    assert!(approx(loss, 0.5798184952529422));
}

#[test]
fn test_cross_entropy_perfect() {
    // Perfect one-hot prediction → loss 0 (terms with target == 0 are skipped).
    let pred = vec![1.0, 0.0, 0.0];
    let target = vec![1.0, 0.0, 0.0];
    let loss = cross_entropy(&pred, &target, 1, 3);
    assert!(approx(loss, 0.0));
}

#[test]
fn test_cross_entropy_single() {
    // -1 · log(0.5) = ln 2
    let pred = vec![0.5];
    let target = vec![1.0];
    let loss = cross_entropy(&pred, &target, 1, 1);
    assert!(approx(loss, 0.6931471805599453));
}

#[test]
fn test_cross_entropy_empty() {
    let loss = cross_entropy(&[], &[], 0, 0);
    assert_eq!(loss, 0.0);
}
