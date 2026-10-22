#[test]
fn test_build_contains() {
    let s: [usize; 3] = [97, 98, 99]; // "abc"
    let sam = string_suffix_automaton::sam_build(&s);
    assert_eq!(sam.sz, 4);

    let p1: [usize; 2] = [97, 98]; // "ab"
    assert_eq!(string_suffix_automaton::sam_contains(&sam, &p1), true);

    let p2: [usize; 2] = [98, 99]; // "bc"
    assert_eq!(string_suffix_automaton::sam_contains(&sam, &p2), true);

    let p3: [usize; 2] = [97, 99]; // "ac" (not a substring)
    assert_eq!(string_suffix_automaton::sam_contains(&sam, &p3), false);

    let p4: [usize; 0] = []; // empty always a substring
    assert_eq!(string_suffix_automaton::sam_contains(&sam, &p4), true);
}

#[test]
fn test_contains_full_and_overlong() {
    let s: [usize; 3] = [97, 98, 97]; // "aba"
    let sam = string_suffix_automaton::sam_build(&s);

    let p1: [usize; 3] = [97, 98, 97]; // "aba"
    assert_eq!(string_suffix_automaton::sam_contains(&sam, &p1), true);

    let p2: [usize; 4] = [97, 98, 97, 98]; // "abab" (longer than s)
    assert_eq!(string_suffix_automaton::sam_contains(&sam, &p2), false);

    let p3: [usize; 1] = [100]; // "d" (char not in s)
    assert_eq!(string_suffix_automaton::sam_contains(&sam, &p3), false);
}

#[test]
fn test_distinct_substrings() {
    // "abc": a, b, c, ab, bc, abc = 6
    let s: [usize; 3] = [97, 98, 99];
    let sam = string_suffix_automaton::sam_build(&s);
    assert_eq!(string_suffix_automaton::sam_distinct_substrings(&sam), 6);

    // "aaa": a, aa, aaa = 3
    let s2: [usize; 3] = [97, 97, 97];
    let sam2 = string_suffix_automaton::sam_build(&s2);
    assert_eq!(string_suffix_automaton::sam_distinct_substrings(&sam2), 3);

    // "aba": a, b, ab, ba, aba = 5
    let s3: [usize; 3] = [97, 98, 97];
    let sam3 = string_suffix_automaton::sam_build(&s3);
    assert_eq!(string_suffix_automaton::sam_distinct_substrings(&sam3), 5);

    // "abab": a, b, ab, ba, aba, bab, abab = 7
    let s4: [usize; 4] = [97, 98, 97, 98];
    let sam4 = string_suffix_automaton::sam_build(&s4);
    assert_eq!(string_suffix_automaton::sam_distinct_substrings(&sam4), 7);
}

#[test]
fn test_distinct_substrings_with_split() {
    // "abbb": a, b, ab, bb, abb, bbb, abbb = 7 (forces a clone)
    let s: [usize; 4] = [97, 98, 98, 98];
    let sam = string_suffix_automaton::sam_build(&s);
    assert_eq!(string_suffix_automaton::sam_distinct_substrings(&sam), 7);
}
