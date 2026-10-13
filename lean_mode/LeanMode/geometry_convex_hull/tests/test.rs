#[test]
fn test_square() {
    let p = [0.0, 0.0, 4.0, 0.0, 4.0, 4.0, 0.0, 4.0];
    let h = geometry_convex_hull::convex_hull(&p, 4);
    // CCW: (0,0), (4,0), (4,4), (0,4).
    assert_eq!(h.len(), 8);
    assert_eq!(h[0], 0.0);
    assert_eq!(h[1], 0.0);
    assert_eq!(h[2], 4.0);
    assert_eq!(h[3], 0.0);
    assert_eq!(h[4], 4.0);
    assert_eq!(h[5], 4.0);
    assert_eq!(h[6], 0.0);
    assert_eq!(h[7], 4.0);
}

#[test]
fn test_square_with_interior_point() {
    let p = [0.0, 0.0, 4.0, 0.0, 4.0, 4.0, 0.0, 4.0, 2.0, 2.0];
    let h = geometry_convex_hull::convex_hull(&p, 5);
    assert_eq!(h.len(), 8);
    assert_eq!(h[0], 0.0);
    assert_eq!(h[1], 0.0);
    assert_eq!(h[2], 4.0);
    assert_eq!(h[3], 0.0);
    assert_eq!(h[4], 4.0);
    assert_eq!(h[5], 4.0);
    assert_eq!(h[6], 0.0);
    assert_eq!(h[7], 4.0);
}

#[test]
fn test_collinear_dropped() {
    // Three collinear points: the middle one is not an extreme vertex.
    let p = [0.0, 0.0, 1.0, 1.0, 2.0, 2.0];
    let h = geometry_convex_hull::convex_hull(&p, 3);
    assert_eq!(h.len(), 4);
    assert_eq!(h[0], 0.0);
    assert_eq!(h[1], 0.0);
    assert_eq!(h[2], 2.0);
    assert_eq!(h[3], 2.0);
}

#[test]
fn test_triangle() {
    let p = [0.0, 0.0, 2.0, 0.0, 0.0, 2.0];
    let h = geometry_convex_hull::convex_hull(&p, 3);
    assert_eq!(h.len(), 6);
    assert_eq!(h[0], 0.0);
    assert_eq!(h[1], 0.0);
    assert_eq!(h[2], 2.0);
    assert_eq!(h[3], 0.0);
    assert_eq!(h[4], 0.0);
    assert_eq!(h[5], 2.0);
}

#[test]
fn test_single_point() {
    let p = [3.0, 4.0];
    let h = geometry_convex_hull::convex_hull(&p, 1);
    assert_eq!(h.len(), 2);
    assert_eq!(h[0], 3.0);
    assert_eq!(h[1], 4.0);
}
