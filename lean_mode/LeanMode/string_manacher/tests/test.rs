#[test]
fn test_manacher_abacaba() {
    let d = string_manacher::manacher(b"abacaba");
    assert_eq!(d.len(), 7);
    assert_eq!(d[0], 1);
    assert_eq!(d[1], 2);
    assert_eq!(d[2], 1);
    assert_eq!(d[3], 4);
    assert_eq!(d[4], 1);
    assert_eq!(d[5], 2);
    assert_eq!(d[6], 1);
}

#[test]
fn test_manacher_all_same() {
    let d = string_manacher::manacher(b"aaa");
    assert_eq!(d.len(), 3);
    assert_eq!(d[0], 1);
    assert_eq!(d[1], 2);
    assert_eq!(d[2], 1);
}

#[test]
fn test_manacher_aba() {
    let d = string_manacher::manacher(b"aba");
    assert_eq!(d.len(), 3);
    assert_eq!(d[0], 1);
    assert_eq!(d[1], 2);
    assert_eq!(d[2], 1);
}

#[test]
fn test_manacher_odd_only() {
    // Even-length palindromes are not reported (odd-only variant).
    let d = string_manacher::manacher(b"abba");
    assert_eq!(d.len(), 4);
    assert_eq!(d[0], 1);
    assert_eq!(d[1], 1);
    assert_eq!(d[2], 1);
    assert_eq!(d[3], 1);
}

#[test]
fn test_manacher_single() {
    let d = string_manacher::manacher(b"x");
    assert_eq!(d.len(), 1);
    assert_eq!(d[0], 1);
}

#[test]
fn test_manacher_empty() {
    let d = string_manacher::manacher(b"");
    assert_eq!(d.len(), 0);
}
