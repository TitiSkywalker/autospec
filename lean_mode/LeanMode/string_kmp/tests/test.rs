#[test]
fn test_prefix_function() {
    let pi = string_kmp::prefix_function(b"aabaab");
    assert_eq!(pi.len(), 6);
    assert_eq!(pi[0], 0);
    assert_eq!(pi[1], 1);
    assert_eq!(pi[2], 0);
    assert_eq!(pi[3], 1);
    assert_eq!(pi[4], 2);
    assert_eq!(pi[5], 3);
}

#[test]
fn test_prefix_function_simple() {
    let pi = string_kmp::prefix_function(b"ababaca");
    assert_eq!(pi.len(), 7);
    assert_eq!(pi[0], 0);
    assert_eq!(pi[1], 0);
    assert_eq!(pi[2], 1);
    assert_eq!(pi[3], 2);
    assert_eq!(pi[4], 3);
    assert_eq!(pi[5], 0);
    assert_eq!(pi[6], 1);
}

#[test]
fn test_kmp_found() {
    let r = string_kmp::kmp_search(b"abcabcabd", b"abcabd");
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 3);
}

#[test]
fn test_kmp_found_at_start() {
    let r = string_kmp::kmp_search(b"hello world", b"hello");
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 0);
}

#[test]
fn test_kmp_not_found() {
    let r = string_kmp::kmp_search(b"hello", b"xyz");
    let is_none = match r {
        Some(_) => false,
        None => true,
    };
    assert!(is_none);
}

#[test]
fn test_kmp_empty_pattern() {
    let r = string_kmp::kmp_search(b"abc", b"");
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 0);
}

#[test]
fn test_kmp_overlapping() {
    // Pattern "aaa" occurs in "aaaaa" at indices 0, 1, 2; KMP reports the first.
    let r = string_kmp::kmp_search(b"aaaaa", b"aaa");
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 0);
}
