use tensor_sigmoid::sigmoid2d;

fn approx(a: f64, b: f64) -> bool {
    let d = if a - b < 0.0 { b - a } else { a - b };
    d < 1e-12
}

#[test]
fn test_sigmoid_matrix() {
    let input = vec![0.0, 1.0, -1.0, 2.5, -3.0, 0.0];
    let out = sigmoid2d(&input, 2, 3);
    assert_eq!(out.len(), 6);
    assert!(approx(out[0], 0.5));
    assert!(approx(out[1], 0.7310585786300049));
    assert!(approx(out[2], 0.2689414213699951));
    assert!(approx(out[3], 0.9241418199787566));
    assert!(approx(out[4], 0.04742587317756678));
    assert!(approx(out[5], 0.5));
}

#[test]
fn test_sigmoid_complement() {
    // σ(x) + σ(-x) = 1
    let input = vec![0.5, -0.5];
    let out = sigmoid2d(&input, 1, 2);
    assert!(approx(out[0] + out[1], 1.0));
}

#[test]
fn test_sigmoid_empty() {
    let out = sigmoid2d(&[], 0, 0);
    assert!(out.is_empty());
    let out2 = sigmoid2d(&[1.0, 2.0], 1, 0);
    assert!(out2.is_empty());
}
