#[test]
fn test_two_points() {
    let p = [2.0, 3.0, 5.0, 7.0];
    // Squared distance = 3² + 4² = 25.
    assert_eq!(geometry_closest_pair::closest_pair(&p, 2), 25.0);
}

#[test]
fn test_three_points() {
    // Closest pair (0,0)-(3,0): squared distance 9.
    let p = [0.0, 0.0, 3.0, 0.0, 10.0, 0.0];
    assert_eq!(geometry_closest_pair::closest_pair(&p, 3), 9.0);
}

#[test]
fn test_unit_square_corners() {
    // Adjacent corners are distance 1 apart: min squared distance 1.
    let p = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    assert_eq!(geometry_closest_pair::closest_pair(&p, 4), 1.0);
}

#[test]
fn test_spread() {
    // Nearest pair is at squared distance 50.
    let p = [0.0, 0.0, 10.0, 0.0, 5.0, 5.0, 10.0, 10.0, 0.0, 10.0];
    assert_eq!(geometry_closest_pair::closest_pair(&p, 5), 50.0);
}

#[test]
fn test_single_point() {
    let p = [1.0, 2.0];
    assert_eq!(geometry_closest_pair::closest_pair(&p, 1), 0.0);
}
