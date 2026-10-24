//! Translated tests for LZ78 dictionary compression.
//!
//! Everything here is checked with *forward walks* only: a factor's word is
//! recognised by walking the trie's child table from the root, and its length
//! is obtained by counting the parent links of its node.  That keeps the tests
//! inside the translatable subset (flat `Vec`s, `while` loops, `usize`).

use string_lzw::Lz78;

/// Flat index for the `256`-wide child table `next[v * 256 + c]`.
fn t_idx(v: usize, c: usize) -> usize {
    let t = v * 256;
    let r = t + c;
    r
}

/// Sum of two `usize`s, used where the two operands are both computed values.
fn t_add(a: usize, b: usize) -> usize {
    let r = a + b;
    r
}

/// Length of the phrase of node `v`: the number of `link` steps to the root.
fn t_depth(t: &Lz78, v0: usize) -> usize {
    let mut d: usize = 0;
    let mut v = v0;
    while v != 0 {
        v = t.link[v];
        d += 1;
    }
    d
}

/// Walk the word `s[pos .. pos + len]` down the child table of `t`; returns the
/// node it ends in, or `nf` when the walk leaves the trie.
fn t_node_of(t: &Lz78, s: &[usize], pos: usize, len: usize, nf: usize) -> usize {
    let mut v = 0;
    let mut i = 0;
    let mut ok = true;
    while i < len && ok {
        let c = s[pos + i];
        let idx = t_idx(v, c);
        let nv = t.next[idx];
        if nv == 0 {
            ok = false;
        } else {
            v = nv;
        }
        i += 1;
    }
    if ok {
        v
    } else {
        nf
    }
}

/// The `(start, length)` of every factor, decoded from the codes.
fn t_factor_spans(t: &Lz78) -> (Vec<usize>, Vec<usize>) {
    let mut starts: Vec<usize> = Vec::new();
    let mut lens: Vec<usize> = Vec::new();
    let mut pos: usize = 0;
    let mut k: usize = 0;
    while k < t.codes.len() {
        let v = t.codes[k];
        let d = t_depth(t, v);
        starts.push(pos);
        lens.push(d);
        pos = t_add(pos, d);
        k += 1;
    }
    (starts, lens)
}

/// Number of factors whose word is not recognised by the trie at the factor's
/// own start position (so `0` means "the codes decode to a tiling of `s` by
/// trie words"), plus `1` when the factors do not cover `s` exactly.
fn t_decode_ok(t: &Lz78, s: &[usize]) -> usize {
    let nf = t.next.len();
    let (starts, lens) = t_factor_spans(t);
    let mut bad: usize = 0;
    let mut pos: usize = 0;
    let mut k: usize = 0;
    while k < starts.len() {
        let v = t.codes[k];
        let node = t_node_of(t, s, starts[k], lens[k], nf);
        if node != v {
            bad += 1;
        }
        pos = t_add(pos, lens[k]);
        k += 1;
    }
    if pos != s.len() {
        bad += 1;
    }
    bad
}

/// Compare the two windows `s[p .. p+l]` and `s[q .. q+l]`.
fn t_same_window(s: &[usize], p: usize, q: usize, l: usize) -> bool {
    let mut same = true;
    let mut i = 0;
    while i < l {
        let x = s[p + i];
        let y = s[q + i];
        if x != y {
            same = false;
        }
        i += 1;
    }
    same
}

/// The number of *distinct* factor words, plus one (the root): the node count a
/// correct LZ78 dictionary must have.
fn t_expected_nodes(t: &Lz78, s: &[usize]) -> usize {
    let (starts, lens) = t_factor_spans(t);
    let mut count: usize = 1;
    let mut k: usize = 0;
    while k < starts.len() {
        let mut first = true;
        let mut m: usize = 0;
        while m < k {
            if lens[m] == lens[k] {
                let same = t_same_window(s, starts[m], starts[k], lens[k]);
                if same {
                    first = false;
                }
            }
            m += 1;
        }
        if first {
            count += 1;
        }
        k += 1;
    }
    count
}

/// Every factor's base — the factor word without its last character, or the
/// whole word when the input ends at the factor — must be a phrase of the trie
/// (the root or an earlier factor word).  Returns the number of violations.
fn t_base_ok(t: &Lz78, s: &[usize]) -> usize {
    let nf = t.next.len();
    let (starts, lens) = t_factor_spans(t);
    let mut bad: usize = 0;
    let mut k: usize = 0;
    while k < lens.len() {
        let start = starts[k];
        let len = lens[k];
        // The base: all but the last character, unless the factor exhausts `s`.
        let rest = s.len() - start;
        let base = if len < rest { len - 1 } else { len };
        if base != 0 {
            let node = t_node_of(t, s, start, base, nf);
            if node == nf {
                bad += 1;
            }
        }
        k += 1;
    }
    bad
}

#[test]
fn test_empty_and_single() {
    let e: [usize; 0] = [];
    let t0 = string_lzw::lz78(&e);
    assert_eq!(t0.sz, 1);
    assert_eq!(t0.codes.len(), 0);

    let s1: [usize; 1] = [97]; // "a"
    let t1 = string_lzw::lz78(&s1);
    assert_eq!(t1.sz, 2);
    assert_eq!(t1.codes.len(), 1);
    assert_eq!(t1.codes[0], 1);
    assert_eq!(t_depth(&t1, 1), 1);
    assert_eq!(t_decode_ok(&t1, &s1), 0);
    assert_eq!(t_base_ok(&t1, &s1), 0);
}

#[test]
fn test_repeated_char() {
    let s: [usize; 3] = [97, 97, 97]; // "aaa"
    let t = string_lzw::lz78(&s);
    // factors: "a" then "aa"
    assert_eq!(t.codes.len(), 2);
    assert_eq!(t.sz, 3);
    assert_eq!(t_depth(&t, t.codes[0]), 1);
    assert_eq!(t_depth(&t, t.codes[1]), 2);
    assert_eq!(t_decode_ok(&t, &s), 0);
    assert_eq!(t_base_ok(&t, &s), 0);
    assert_eq!(t.sz, t_expected_nodes(&t, &s));
}

#[test]
fn test_two_letters() {
    let s1: [usize; 4] = [97, 98, 97, 98]; // "abab"
    let t1 = string_lzw::lz78(&s1);
    // factors: "a", "b", "ab"
    assert_eq!(t1.codes.len(), 3);
    assert_eq!(t1.sz, 4);
    assert_eq!(t_decode_ok(&t1, &s1), 0);
    assert_eq!(t_base_ok(&t1, &s1), 0);
    assert_eq!(t1.sz, t_expected_nodes(&t1, &s1));

    let s2: [usize; 5] = [97, 98, 99, 97, 98]; // "abcab"
    let t2 = string_lzw::lz78(&s2);
    // factors: "a", "b", "c", "ab"
    assert_eq!(t2.codes.len(), 4);
    assert_eq!(t2.sz, 5);
    assert_eq!(t_decode_ok(&t2, &s2), 0);
    assert_eq!(t_base_ok(&t2, &s2), 0);
    assert_eq!(t2.sz, t_expected_nodes(&t2, &s2));
}

#[test]
fn test_trailing_phrase_is_reused() {
    // "aaa" factors as "a", "aa": the trailing "aa" needs no further character,
    // so the second factor is exactly the phrase "aa".
    let s: [usize; 3] = [97, 97, 97];
    let t = string_lzw::lz78(&s);
    let (starts, lens) = t_factor_spans(&t);
    assert_eq!(starts[0], 0);
    assert_eq!(lens[0], 1);
    assert_eq!(starts[1], 1);
    assert_eq!(lens[1], 2);
    assert_eq!(t_add(starts[1], lens[1]), s.len());
}

#[test]
fn test_banana_and_mississippi() {
    let s1: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let t1 = string_lzw::lz78(&s1);
    assert_eq!(t_decode_ok(&t1, &s1), 0);
    assert_eq!(t_base_ok(&t1, &s1), 0);
    assert_eq!(t1.sz, t_expected_nodes(&t1, &s1));

    let s2: [usize; 11] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105];
    // "mississippi" factors as "m", "i", "s", "si", "ss", "ip", "p", "i" —
    // the trailing "i" is an existing phrase, so only seven nodes are created.
    let t2 = string_lzw::lz78(&s2);
    assert_eq!(t2.codes.len(), 8);
    assert_eq!(t2.sz, 8);
    assert_eq!(t_decode_ok(&t2, &s2), 0);
    assert_eq!(t_base_ok(&t2, &s2), 0);
    assert_eq!(t2.sz, t_expected_nodes(&t2, &s2));
}

#[test]
fn test_periodic_string() {
    let s: [usize; 8] = [97, 98, 97, 98, 97, 98, 97, 98]; // "abababab"
    let t = string_lzw::lz78(&s);
    // factors: "a", "b", "ab", "aba", then the trailing "ab" (an existing
    // phrase: the remaining suffix is exactly a phrase, with no character left
    // to extend it by).
    assert_eq!(t.codes.len(), 5);
    assert_eq!(t.sz, 5);
    assert_eq!(t_decode_ok(&t, &s), 0);
    assert_eq!(t_base_ok(&t, &s), 0);
    assert_eq!(t.sz, t_expected_nodes(&t, &s));
}
