//! Tests for `two_way_find`, cross-checked against a brute-force reference.

/// Brute-force reference: all starting positions of `p` in `t`, in order.
fn brute_occ(t: &[usize], p: &[usize]) -> Vec<usize> {
    let n = t.len();
    let m = p.len();
    let mut out: Vec<usize> = Vec::new();
    if m == 0 {
        return out;
    }
    if m > n {
        return out;
    }
    let mut i: usize = 0;
    while i + m <= n {
        let mut j: usize = 0;
        let mut ok: bool = true;
        while j < m {
            let a = t[i + j];
            let b = p[j];
            if a != b {
                ok = false;
            }
            j += 1;
        }
        if ok {
            out.push(i);
        }
        i += 1;
    }
    out
}

/// Assert that `two_way_find` and the brute-force reference agree, element by
/// element.
fn check_occ(t: &[usize], p: &[usize]) {
    let got = string_two_way::two_way_find(t, p);
    let exp = brute_occ(t, p);
    let lg = got.len();
    let le = exp.len();
    assert_eq!(lg, le);
    let mut k: usize = 0;
    while k < lg {
        let a = got[k];
        let b = exp[k];
        assert_eq!(a, b);
        k += 1;
    }
}

#[test]
fn test_empty_pattern() {
    let text: [usize; 3] = [97, 98, 99];
    let pat: [usize; 0] = [];
    check_occ(&text, &pat);
}

#[test]
fn test_empty_text() {
    let text: [usize; 0] = [];
    let pat: [usize; 3] = [97, 98, 99];
    check_occ(&text, &pat);
}

#[test]
fn test_both_empty() {
    let text: [usize; 0] = [];
    let pat: [usize; 0] = [];
    check_occ(&text, &pat);
}

#[test]
fn test_pattern_equals_text() {
    // "abcabc" occurs in "abcabc" exactly once, at position 0.
    let text: [usize; 6] = [97, 98, 99, 97, 98, 99];
    let pat: [usize; 6] = [97, 98, 99, 97, 98, 99];
    check_occ(&text, &pat);
}

#[test]
fn test_pattern_longer_than_text() {
    let text: [usize; 2] = [97, 98];
    let pat: [usize; 3] = [97, 98, 99];
    check_occ(&text, &pat);
}

#[test]
fn test_periodic() {
    // "ababab" in "ababababab": positions 0, 2, 4.
    let text: [usize; 10] = [97, 98, 97, 98, 97, 98, 97, 98, 97, 98];
    let pat: [usize; 6] = [97, 98, 97, 98, 97, 98];
    check_occ(&text, &pat);
}

#[test]
fn test_worst_case_all_equal() {
    // The classical worst case for naive matching: 12 a's searched for 5 a's.
    let text: [usize; 12] = [97; 12];
    let pat: [usize; 5] = [97; 5];
    check_occ(&text, &pat);
}

#[test]
fn test_long_border() {
    // "abcab" has the border "ab": it occurs in "abcabcabc" at 0 and at 3.
    let text: [usize; 9] = [97, 98, 99, 97, 98, 99, 97, 98, 99];
    let pat: [usize; 5] = [97, 98, 99, 97, 98];
    check_occ(&text, &pat);
}

#[test]
fn test_no_match() {
    let text: [usize; 5] = [104, 101, 108, 108, 111];
    let pat: [usize; 3] = [120, 121, 122];
    check_occ(&text, &pat);
}

#[test]
fn test_single_character() {
    let text: [usize; 4] = [97, 98, 97, 97];
    let pat: [usize; 1] = [97];
    check_occ(&text, &pat);
}

#[test]
fn test_expected_values() {
    // "ababab" in "ababababab" occurs at 0, 2 and 4.
    let text: [usize; 10] = [97, 98, 97, 98, 97, 98, 97, 98, 97, 98];
    let pat: [usize; 6] = [97, 98, 97, 98, 97, 98];
    let got = string_two_way::two_way_find(&text, &pat);
    assert_eq!(got.len(), 3);
    assert_eq!(got[0], 0);
    assert_eq!(got[1], 2);
    assert_eq!(got[2], 4);

    // "aaa" in eight a's occurs six times.
    let text2: [usize; 8] = [97; 8];
    let pat2: [usize; 3] = [97; 3];
    let got2 = string_two_way::two_way_find(&text2, &pat2);
    assert_eq!(got2.len(), 6);
    assert_eq!(got2[0], 0);
    assert_eq!(got2[5], 5);

    // "abcabd" in "abcabcabd" occurs once, at position 3.
    let text3: [usize; 9] = [97, 98, 99, 97, 98, 99, 97, 98, 100];
    let pat3: [usize; 6] = [97, 98, 99, 97, 98, 100];
    let got3 = string_two_way::two_way_find(&text3, &pat3);
    assert_eq!(got3.len(), 1);
    assert_eq!(got3[0], 3);
}

#[test]
fn test_cross_check_battery() {
    // A battery of patterns with short periods, long periods, borders and
    // overlaps, each cross-checked against the brute-force reference.
    let acba: [usize; 4] = [97, 99, 98, 97];
    let t12: [usize; 12] = [97, 99, 98, 97, 97, 99, 98, 97, 97, 99, 98, 97];
    check_occ(&t12, &acba);

    // Fibonacci word "abaababaabaab" and its factors.
    let fib: [usize; 13] = [97, 98, 97, 97, 98, 97, 98, 97, 97, 98, 97, 97, 98];
    let f1: [usize; 5] = [97, 98, 97, 97, 98];
    let f2: [usize; 6] = [97, 98, 97, 97, 98, 97];
    let f3: [usize; 8] = [97, 98, 97, 97, 98, 97, 98, 97];
    check_occ(&fib, &f1);
    check_occ(&fib, &f2);
    check_occ(&fib, &f3);

    // "abaaba" is a square-like pattern with period 3 in a ternary text.
    let t18: [usize; 18] = [
        97, 98, 97, 97, 98, 97, 99, 97, 98, 97, 97, 98, 97, 97, 98, 97, 97, 98,
    ];
    let ab: [usize; 6] = [97, 98, 97, 97, 98, 97];
    check_occ(&t18, &ab);

    // A pattern that only matches near the end of the text.
    let t13: [usize; 13] = [97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 98, 97, 98];
    let tail: [usize; 4] = [98, 97, 97, 98];
    check_occ(&t13, &tail);
    let tail2: [usize; 3] = [97, 98, 97];
    check_occ(&t13, &tail2);

    // Alternating text with an alternating pattern (long border, period 2).
    let t16: [usize; 16] = [
        97, 98, 97, 98, 97, 98, 97, 98, 97, 98, 97, 98, 97, 98, 97, 98,
    ];
    let alt4: [usize; 4] = [97, 98, 97, 98];
    check_occ(&t16, &alt4);

    // A pattern with no self-overlap inside a text that contains many of its
    // rotations.
    let t21: [usize; 21] = [
        97, 98, 99, 97, 98, 99, 97, 98, 99, 97, 98, 99, 97, 98, 99, 97, 98, 99, 97, 98, 99,
    ];
    let three: [usize; 3] = [97, 98, 99];
    check_occ(&t21, &three);
    let bca: [usize; 3] = [98, 99, 97];
    check_occ(&t21, &bca);
    let long: [usize; 9] = [97, 98, 99, 97, 98, 99, 97, 98, 99];
    check_occ(&t21, &long);

    // Some texts shorter than the pattern, and a pattern of length one.
    let one: [usize; 1] = [99];
    check_occ(&three, &one);
    check_occ(&t21, &one);
    let zero: [usize; 1] = [0];
    check_occ(&t21, &zero);

    // A single-character text.
    let single: [usize; 1] = [97];
    check_occ(&single, &single);
    check_occ(&single, &one);
}

#[test]
fn test_exhaustive_binary() {
    // Every binary text of length at most seven (255 of them) against every
    // binary pattern of length at most five (63 of them): 16065 brute-force
    // cross-checks.  Words are enumerated with a base-two counter built from
    // `/` and `%` (no bit tricks, so the translation stays in the subset that
    // AENEAS supports).
    let mut tl: usize = 0;
    while tl <= 7 {
        let mut tcap: usize = 1;
        let mut e: usize = 0;
        while e < tl {
            tcap = tcap * 2;
            e += 1;
        }
        let mut a: usize = 0;
        while a < tcap {
            let mut texto: Vec<usize> = Vec::new();
            let mut x = a;
            let mut k: usize = 0;
            while k < tl {
                let bit = x % 2;
                let c = 97 + bit;
                texto.push(c);
                x = x / 2;
                k += 1;
            }
            let mut pl: usize = 0;
            while pl <= 5 {
                let mut pcap: usize = 1;
                let mut e2: usize = 0;
                while e2 < pl {
                    pcap = pcap * 2;
                    e2 += 1;
                }
                let mut b: usize = 0;
                while b < pcap {
                    let mut pat: Vec<usize> = Vec::new();
                    let mut y = b;
                    let mut k2: usize = 0;
                    while k2 < pl {
                        let bit2 = y % 2;
                        let c2 = 97 + bit2;
                        pat.push(c2);
                        y = y / 2;
                        k2 += 1;
                    }
                    check_occ(&texto, &pat);
                    b += 1;
                }
                pl += 1;
            }
            a += 1;
        }
        tl += 1;
    }
}
