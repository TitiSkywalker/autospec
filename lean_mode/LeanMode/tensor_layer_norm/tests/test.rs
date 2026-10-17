use tensor_layer_norm::layer_norm;

fn approx(a: f64, b: f64) -> bool {
    let d = if a - b < 0.0 { b - a } else { a - b };
    d < 1e-9
}

#[test]
fn test_layer_norm_identity_affine() {
    // γ = 1, β = 0 → output is the plain normalized row x̂.
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let gamma = vec![1.0, 1.0, 1.0];
    let beta = vec![0.0, 0.0, 0.0];
    let out = layer_norm(&input, 2, 3, &gamma, &beta, 1e-5);
    assert_eq!(out.len(), 6);
    assert!(approx(out[0], -1.2247356859083902));
    assert!(approx(out[1], 0.0));
    assert!(approx(out[2], 1.2247356859083902));
    assert!(approx(out[3], -1.2247356859083902));
    assert!(approx(out[4], 0.0));
    assert!(approx(out[5], 1.2247356859083902));
}

#[test]
fn test_layer_norm_affine() {
    let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let gamma = vec![2.0, 3.0, 4.0];
    let beta = vec![1.0, -1.0, 0.5];
    let out = layer_norm(&input, 2, 3, &gamma, &beta, 1e-5);
    assert!(approx(out[0], -1.4494713718167804));
    assert!(approx(out[1], -1.0));
    assert!(approx(out[2], 5.398942743633561));
    assert!(approx(out[3], -1.4494713718167804));
    assert!(approx(out[4], -1.0));
    assert!(approx(out[5], 5.398942743633561));
}

#[test]
fn test_layer_norm_zero_mean() {
    // With γ = 1, β = 0, each output row is centered: its mean is 0.
    let input = vec![3.0, 1.0, 4.0, 1.5, 5.0, 2.5];
    let gamma = vec![1.0, 1.0, 1.0];
    let beta = vec![0.0, 0.0, 0.0];
    let out = layer_norm(&input, 2, 3, &gamma, &beta, 1e-5);
    for r in 0..2 {
        let base = r * 3;
        let mut s = 0.0;
        for c in 0..3 {
            s += out[base + c];
        }
        assert!(approx(s / 3.0, 0.0));
    }
}

#[test]
fn test_layer_norm_empty() {
    let out = layer_norm(&[], 0, 0, &[], &[], 1e-5);
    assert!(out.is_empty());
    let out2 = layer_norm(&[1.0, 2.0], 1, 0, &[], &[], 1e-5);
    assert!(out2.is_empty());
}
