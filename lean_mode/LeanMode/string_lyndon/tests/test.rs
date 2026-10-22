#[test]
fn test_empty() {
    let s: [u8; 0] = [];
    let l = string_lyndon::lyndon_factorization(&s);
    assert_eq!(l.len(), 0);
}

#[test]
fn test_single() {
    let s: [u8; 1] = [97]; // "a"
    let l = string_lyndon::lyndon_factorization(&s);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0], 1);
}

#[test]
fn test_aba() {
    let s: [u8; 3] = [97, 98, 97]; // "aba" -> "ab" | "a"
    let l = string_lyndon::lyndon_factorization(&s);
    assert_eq!(l.len(), 2);
    assert_eq!(l[0], 2);
    assert_eq!(l[1], 1);
}

#[test]
fn test_abab() {
    let s: [u8; 4] = [97, 98, 97, 98]; // "abab" -> "ab" | "ab"
    let l = string_lyndon::lyndon_factorization(&s);
    assert_eq!(l.len(), 2);
    assert_eq!(l[0], 2);
    assert_eq!(l[1], 2);
}

#[test]
fn test_aab() {
    let s: [u8; 3] = [97, 97, 98]; // "aab" is itself a Lyndon word
    let l = string_lyndon::lyndon_factorization(&s);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0], 3);
}

#[test]
fn test_abcabc() {
    let s: [u8; 6] = [97, 98, 99, 97, 98, 99]; // "abcabc" -> "abc" | "abc"
    let l = string_lyndon::lyndon_factorization(&s);
    assert_eq!(l.len(), 2);
    assert_eq!(l[0], 3);
    assert_eq!(l[1], 3);
}

#[test]
fn test_bbab() {
    let s: [u8; 4] = [98, 98, 97, 98]; // "bbab" -> "b" | "b" | "ab"
    let l = string_lyndon::lyndon_factorization(&s);
    assert_eq!(l.len(), 3);
    assert_eq!(l[0], 1);
    assert_eq!(l[1], 1);
    assert_eq!(l[2], 2);
}
