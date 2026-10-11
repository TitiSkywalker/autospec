#[test]
fn test_found() {
    let text: [u64; 11] = [104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100];
    let pat: [u64; 3] = [108, 108, 111];
    let r = string_rabin_karp::rabin_karp(&text, &pat);
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 2);
}

#[test]
fn test_found_at_start() {
    let text: [u64; 5] = [97, 98, 99, 100, 101];
    let pat: [u64; 3] = [97, 98, 99];
    let r = string_rabin_karp::rabin_karp(&text, &pat);
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 0);
}

#[test]
fn test_not_found() {
    let text: [u64; 5] = [97, 98, 99, 100, 101];
    let pat: [u64; 2] = [120, 121];
    let r = string_rabin_karp::rabin_karp(&text, &pat);
    let is_none = match r {
        Some(_) => false,
        None => true,
    };
    assert!(is_none);
}

#[test]
fn test_empty_pattern() {
    let text: [u64; 3] = [97, 98, 99];
    let pat: [u64; 0] = [];
    let r = string_rabin_karp::rabin_karp(&text, &pat);
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 0);
}

#[test]
fn test_pattern_longer_than_text() {
    let text: [u64; 2] = [97, 98];
    let pat: [u64; 3] = [97, 98, 99];
    let r = string_rabin_karp::rabin_karp(&text, &pat);
    let is_none = match r {
        Some(_) => false,
        None => true,
    };
    assert!(is_none);
}

#[test]
fn test_first_of_repeated() {
    // "aaaa" in "aaaaaa" occurs first at index 0.
    let text: [u64; 6] = [97, 97, 97, 97, 97, 97];
    let pat: [u64; 4] = [97, 97, 97, 97];
    let r = string_rabin_karp::rabin_karp(&text, &pat);
    let idx = match r {
        Some(i) => i,
        None => 999,
    };
    assert_eq!(idx, 0);
}
