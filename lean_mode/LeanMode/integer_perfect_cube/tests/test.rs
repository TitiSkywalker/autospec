#[test]
fn test_perfect_cubes() {
    assert!(integer_perfect_cube::is_perfect_cube(0));
    assert!(integer_perfect_cube::is_perfect_cube(1));
    assert!(integer_perfect_cube::is_perfect_cube(8));
    assert!(integer_perfect_cube::is_perfect_cube(27));
    assert!(integer_perfect_cube::is_perfect_cube(64));
    assert!(integer_perfect_cube::is_perfect_cube(125));
    assert!(integer_perfect_cube::is_perfect_cube(1000));
}

#[test]
fn test_non_perfect_cubes() {
    assert!(!integer_perfect_cube::is_perfect_cube(2));
    assert!(!integer_perfect_cube::is_perfect_cube(3));
    assert!(!integer_perfect_cube::is_perfect_cube(7));
    assert!(!integer_perfect_cube::is_perfect_cube(9));
    assert!(!integer_perfect_cube::is_perfect_cube(26));
    assert!(!integer_perfect_cube::is_perfect_cube(28));
    assert!(!integer_perfect_cube::is_perfect_cube(999));
    assert!(!integer_perfect_cube::is_perfect_cube(1001));
}
