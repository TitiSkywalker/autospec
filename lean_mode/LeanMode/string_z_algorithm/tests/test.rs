#[test]
fn test_z_all_same() {
    let z = string_z_algorithm::z_algorithm(b"aaaaa");
    assert_eq!(z.len(), 5);
    assert_eq!(z[0], 5);
    assert_eq!(z[1], 4);
    assert_eq!(z[2], 3);
    assert_eq!(z[3], 2);
    assert_eq!(z[4], 1);
}

#[test]
fn test_z_abababa() {
    let z = string_z_algorithm::z_algorithm(b"abababa");
    assert_eq!(z.len(), 7);
    assert_eq!(z[0], 7);
    assert_eq!(z[1], 0);
    assert_eq!(z[2], 5);
    assert_eq!(z[3], 0);
    assert_eq!(z[4], 3);
    assert_eq!(z[5], 0);
    assert_eq!(z[6], 1);
}

#[test]
fn test_z_aaabaab() {
    let z = string_z_algorithm::z_algorithm(b"aaabaab");
    assert_eq!(z.len(), 7);
    assert_eq!(z[0], 7);
    assert_eq!(z[1], 2);
    assert_eq!(z[2], 1);
    assert_eq!(z[3], 0);
    assert_eq!(z[4], 2);
    assert_eq!(z[5], 1);
    assert_eq!(z[6], 0);
}

#[test]
fn test_z_single() {
    let z = string_z_algorithm::z_algorithm(b"x");
    assert_eq!(z.len(), 1);
    assert_eq!(z[0], 1);
}

#[test]
fn test_z_empty() {
    let z = string_z_algorithm::z_algorithm(b"");
    assert_eq!(z.len(), 0);
}
