#[test]
fn test_perfect_squares() {
    assert!(integer_perfect_square::is_perfect_square(0));
    assert!(integer_perfect_square::is_perfect_square(1));
    assert!(integer_perfect_square::is_perfect_square(4));
    assert!(integer_perfect_square::is_perfect_square(9));
    assert!(integer_perfect_square::is_perfect_square(16));
    assert!(integer_perfect_square::is_perfect_square(81));
    assert!(integer_perfect_square::is_perfect_square(10000));
}

#[test]
fn test_non_perfect_squares() {
    assert!(!integer_perfect_square::is_perfect_square(2));
    assert!(!integer_perfect_square::is_perfect_square(3));
    assert!(!integer_perfect_square::is_perfect_square(8));
    assert!(!integer_perfect_square::is_perfect_square(99));
    assert!(!integer_perfect_square::is_perfect_square(10001));
}
