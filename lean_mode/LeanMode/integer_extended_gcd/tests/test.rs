#[test]
fn test_basic_1() {
    let (g, x, y) = integer_extended_gcd::extended_gcd(101, 13);
    assert_eq!(g, 1);
    assert_eq!(x, 4);
    assert_eq!(y, -31);
}

#[test]
fn test_basic_2() {
    let (g, x, y) = integer_extended_gcd::extended_gcd(123, 19);
    assert_eq!(g, 1);
    assert_eq!(x, -2);
    assert_eq!(y, 13);
}

#[test]
fn test_basic_3() {
    let (g, x, y) = integer_extended_gcd::extended_gcd(25, 36);
    assert_eq!(g, 1);
    assert_eq!(x, 13);
    assert_eq!(y, -9);
}

#[test]
fn test_composite_gcd() {
    let (g, x, y) = integer_extended_gcd::extended_gcd(69, 54);
    assert_eq!(g, 3);
    assert_eq!(x, -7);
    assert_eq!(y, 9);
}

#[test]
fn test_large() {
    let (g, x, y) = integer_extended_gcd::extended_gcd(55, 79);
    assert_eq!(g, 1);
    assert_eq!(x, 23);
    assert_eq!(y, -16);
}

#[test]
fn test_common_factor() {
    let (g, x, y) = integer_extended_gcd::extended_gcd(50, 70);
    assert_eq!(g, 10);
    assert_eq!(x, 3);
    assert_eq!(y, -2);
}
