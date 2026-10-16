#[test]
fn test_two_matrices() {
    // A(10×20) · B(20×30) = 10*20*30 = 6000
    let dims = vec![10, 20, 30];
    assert_eq!(dp_interval_dp::matrix_chain_order(&dims), 6000);
}

#[test]
fn test_three_matrices() {
    // (AB)C = 10*20*30 + 10*30*40 = 6000 + 12000 = 18000
    // A(BC) = 20*30*40 + 10*20*40 = 24000 + 8000 = 32000
    let dims = vec![10, 20, 30, 40];
    assert_eq!(dp_interval_dp::matrix_chain_order(&dims), 18000);
}

#[test]
fn test_four_matrices_classic() {
    // A(40×20) B(20×30) C(30×10) D(10×30); optimal (A(BC))D = 26000
    let dims = vec![40, 20, 30, 10, 30];
    assert_eq!(dp_interval_dp::matrix_chain_order(&dims), 26000);
}

#[test]
fn test_four_matrices_2() {
    // A(5×4) B(4×6) C(6×2) D(2×7); optimal (A(BC))D = 158
    let dims = vec![5, 4, 6, 2, 7];
    assert_eq!(dp_interval_dp::matrix_chain_order(&dims), 158);
}

#[test]
fn test_single_matrix() {
    let dims = vec![1, 2];
    assert_eq!(dp_interval_dp::matrix_chain_order(&dims), 0);
}

#[test]
fn test_empty_dims() {
    let dims: Vec<usize> = vec![];
    assert_eq!(dp_interval_dp::matrix_chain_order(&dims), 0);
}

#[test]
fn test_single_dim() {
    // No matrices (a single scalar dimension) — zero multiplications.
    let dims = vec![5];
    assert_eq!(dp_interval_dp::matrix_chain_order(&dims), 0);
}
