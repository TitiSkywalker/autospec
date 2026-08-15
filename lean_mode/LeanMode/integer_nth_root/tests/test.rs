#[test]
fn test_small() {
    assert_eq!(integer_nth_root::nth_root(0, 3), 0);
    assert_eq!(integer_nth_root::nth_root(1, 5), 1);
    assert_eq!(integer_nth_root::nth_root(2, 10), 1);
    assert_eq!(integer_nth_root::nth_root(7, 3), 1);
}

#[test]
fn test_cubes() {
    assert_eq!(integer_nth_root::nth_root(26, 3), 2);
    assert_eq!(integer_nth_root::nth_root(27, 3), 3);
    assert_eq!(integer_nth_root::nth_root(28, 3), 3);
    assert_eq!(integer_nth_root::nth_root(100, 3), 4);
    assert_eq!(integer_nth_root::nth_root(1000, 3), 10);
}

#[test]
fn test_other_roots() {
    assert_eq!(integer_nth_root::nth_root(15, 2), 3);
    assert_eq!(integer_nth_root::nth_root(16, 2), 4);
    assert_eq!(integer_nth_root::nth_root(100, 4), 3);
    assert_eq!(integer_nth_root::nth_root(1024, 10), 2);
    assert_eq!(integer_nth_root::nth_root(12345, 1), 12345);
}
