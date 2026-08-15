#[test]
fn test_basic() {
    assert_eq!(integer_kth_factor::kth_factor(12, 3), 3);
    assert_eq!(integer_kth_factor::kth_factor(7, 2), 7);
    assert_eq!(integer_kth_factor::kth_factor(950, 5), 19);
}

#[test]
fn test_out_of_range() {
    assert_eq!(integer_kth_factor::kth_factor(4, 4), -1);
    assert_eq!(integer_kth_factor::kth_factor(1, 2), -1);
}
