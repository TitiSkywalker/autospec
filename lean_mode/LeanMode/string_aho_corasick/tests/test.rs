#[test]
fn test_aho_multi() {
    // patterns: "ab" (p0), "bca" (p1), "xyz" (p2)
    let patterns: [usize; 8] = [97, 98, 98, 99, 97, 120, 121, 122];
    let offsets: [usize; 4] = [0, 2, 5, 8];
    let text: [usize; 5] = [97, 98, 99, 97, 98]; // "abcab"
    let m = string_aho_corasick::aho_corasick(&text, &patterns, &offsets);
    assert_eq!(m.len(), 3);
    assert_eq!(m[0], true);
    assert_eq!(m[1], true);
    assert_eq!(m[2], false);
}

#[test]
fn test_aho_classic() {
    // patterns: "he" (p0), "she" (p1), "his" (p2), "hers" (p3); text "ushers"
    let patterns: [usize; 12] = [104, 101, 115, 104, 101, 104, 105, 115, 104, 101, 114, 115];
    let offsets: [usize; 5] = [0, 2, 5, 8, 12];
    let text: [usize; 6] = [117, 115, 104, 101, 114, 115];
    let m = string_aho_corasick::aho_corasick(&text, &patterns, &offsets);
    assert_eq!(m.len(), 4);
    assert_eq!(m[0], true); // he
    assert_eq!(m[1], true); // she
    assert_eq!(m[2], false); // his
    assert_eq!(m[3], true); // hers
}

#[test]
fn test_aho_empty_pattern() {
    // patterns: "" (p0), "ab" (p1)
    let patterns: [usize; 2] = [97, 98];
    let offsets: [usize; 3] = [0, 0, 2];
    let text: [usize; 3] = [97, 98, 99]; // "abc"
    let m = string_aho_corasick::aho_corasick(&text, &patterns, &offsets);
    assert_eq!(m.len(), 2);
    assert_eq!(m[0], true); // empty always matches
    assert_eq!(m[1], true); // "ab"
}

#[test]
fn test_aho_no_match() {
    let patterns: [usize; 2] = [120, 121]; // "xy"
    let offsets: [usize; 2] = [0, 2];
    let text: [usize; 3] = [97, 98, 99]; // "abc"
    let m = string_aho_corasick::aho_corasick(&text, &patterns, &offsets);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0], false);
}
