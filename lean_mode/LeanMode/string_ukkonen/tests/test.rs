//! Tests for the suffix tree (trie of all suffixes), against brute-force
//! references.
//!
//! The child table is `nodes * 256` wide, so brute-force checks are run on the
//! short strings and only structural checks on `mississippi`.

use string_ukkonen::SuffixTree;

/// Flat index for the `256`-wide child table `next[v * 256 + c]`.
fn tidx(v: usize, c: usize) -> usize {
    let t = v * 256;
    let r = t + c;
    r
}

/// The distinct byte values occurring in `s`, in order of first occurrence.
fn t_alphabet(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut alpha: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        let c = s[i];
        let mut seen = false;
        let mut k = 0;
        while k < alpha.len() {
            let d = alpha[k];
            if d == c {
                seen = true;
            }
            k += 1;
        }
        if !seen {
            alpha.push(c);
        }
        i += 1;
    }
    alpha
}

/// Walk the word `buf[base .. base + len]` from the root.  Returns the node it
/// ends in, or `nf` when the word does not occur.
fn t_walk_buf(next: &[usize], buf: &[usize], base: usize, len: usize, nf: usize) -> usize {
    let mut v = 0;
    let mut i = 0;
    let mut ok = true;
    while i < len && ok {
        let c = buf[base + i];
        let idx = tidx(v, c);
        let nv = next[idx];
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

/// Whether the length-`m` window `buf[base .. base + m]` occurs as a window of
/// `s`.
fn t_occurs_buf(s: &[usize], buf: &[usize], base: usize, m: usize) -> bool {
    let n = s.len();
    let mut found = false;
    if m <= n {
        let mut i = 0;
        while i + m <= n && !found {
            let mut k = 0;
            let mut same = true;
            while k < m {
                let x = s[i + k];
                let y = buf[base + k];
                if x != y {
                    same = false;
                }
                k += 1;
            }
            if same {
                found = true;
            }
            i += 1;
        }
    }
    found
}

/// Brute-force number of distinct nonempty substrings of `s`, deduplicated by
/// explicit comparison.
fn t_distinct_count(s: &[usize]) -> usize {
    let n = s.len();
    let mut starts: Vec<usize> = Vec::new();
    let mut lens: Vec<usize> = Vec::new();
    let mut count: usize = 0;
    let mut l = 1;
    while l <= n {
        let mut i = 0;
        while i + l <= n {
            let mut matches: usize = 0;
            let mut k = 0;
            while k < starts.len() {
                let lk = lens[k];
                if lk == l {
                    let sk = starts[k];
                    let mut d = 0;
                    let mut same = true;
                    while d < l {
                        let x = s[sk + d];
                        let y = s[i + d];
                        if x != y {
                            same = false;
                        }
                        d += 1;
                    }
                    if same {
                        matches += 1;
                    }
                }
                k += 1;
            }
            if matches == 0 {
                starts.push(i);
                lens.push(l);
                count += 1;
            }
            i += 1;
        }
        l += 1;
    }
    count
}

/// How many `(start, length)` substrings of `s` are NOT reachable from the root.
fn t_missed_substrings(s: &[usize], t: &SuffixTree) -> usize {
    let n = s.len();
    let nf = t.next.len();
    let mut missed: usize = 0;
    let mut l = 1;
    while l <= n {
        let mut i = 0;
        while i + l <= n {
            let node = t_walk_buf(&t.next, s, i, l, nf);
            if node == nf {
                missed += 1;
            }
            i += 1;
        }
        l += 1;
    }
    missed
}

/// Check every node's suffix link against the brute-force definition: `link[v]`
/// must be the node spelling the longest proper suffix of `v`'s path string
/// that still occurs in `s` (hence is a node).  Returns the number of
/// mismatches.
fn t_links_mismatch(s: &[usize], t: &SuffixTree) -> usize {
    let n = s.len();
    let width = n + 1;
    let sz = t.sz;
    let size = sz * width;
    let alpha = t_alphabet(s);
    // Flattened path strings: node `v`'s path is `path[v * width .. + depth[v]]`.
    let mut path: Vec<usize> = vec![0usize; size];
    let mut depth: Vec<usize> = vec![0usize; sz];
    let mut queue: Vec<usize> = vec![0usize; sz];
    let mut head: usize = 0;
    let mut tail: usize = 1;

    // BFS from the root; a node's children can only be on characters of `s`.
    while head < tail {
        let u = queue[head];
        head += 1;
        let du = depth[u];
        let mut ai = 0;
        while ai < alpha.len() {
            let c = alpha[ai];
            let idx = tidx(u, c);
            let child = t.next[idx];
            if child != 0 {
                let d1 = du + 1;
                depth[child] = d1;
                let mut k = 0;
                while k < du {
                    let src = u * width + k;
                    let x = path[src];
                    let dst = child * width + k;
                    path[dst] = x;
                    k += 1;
                }
                let dst2 = child * width + du;
                path[dst2] = c;
                queue[tail] = child;
                tail += 1;
            }
            ai += 1;
        }
    }

    let nf = size;
    let mut bad: usize = 0;
    let mut v = 1;
    while v < sz {
        let dv = depth[v];
        // The longest proper suffix of `v`'s path that still occurs in `s`.
        let mut best: usize = 0;
        let mut cl: usize = 1;
        while cl < dv {
            let dvc = dv - cl;
            let rwide = v * width;
            let base = rwide + dvc;
            let occurs = t_occurs_buf(s, &path, base, cl);
            if occurs {
                let node = t_walk_buf(&t.next, &path, base, cl, nf);
                best = node;
            }
            cl += 1;
        }
        let lk = t.link[v];
        if lk != best {
            bad += 1;
        }
        v += 1;
    }
    bad
}

/// Full brute-force check of `t`: substring recognition, node count and suffix
/// links.  `0` iff all pass.  Use on the short strings only.
fn t_full_ok(s: &[usize], t: &SuffixTree) -> usize {
    let mut bad: usize = 0;
    let missed = t_missed_substrings(s, t);
    if missed != 0 {
        bad += 1;
    }
    let links = t_links_mismatch(s, t);
    if links != 0 {
        bad += 1;
    }
    let d = t_distinct_count(s);
    let expect = d + 1;
    if t.sz != expect {
        bad += 1;
    }
    bad
}

/// `1` when the word `w` occurs in `t` (walks from the root), else `0`.
fn t_word_ok(t: &SuffixTree, w: &[usize]) -> usize {
    let nf = t.next.len();
    let m = w.len();
    let node = t_walk_buf(&t.next, w, 0, m, nf);
    if node == nf {
        0
    } else {
        1
    }
}

/// `0` when every single-character child of the root links to the root.
fn t_single_char_links_ok(s: &[usize], t: &SuffixTree) -> usize {
    let alpha = t_alphabet(s);
    let mut bad: usize = 0;
    let mut k = 0;
    while k < alpha.len() {
        let c = alpha[k];
        let child = t.next[c]; // root's child on `c`
        let lk = t.link[child];
        if lk != 0 {
            bad += 1;
        }
        k += 1;
    }
    bad
}

#[test]
fn test_empty_tree() {
    let s: [usize; 0] = [];
    let t = string_ukkonen::suffix_tree(&s);
    assert_eq!(t.sz, 1);
    assert_eq!(t.link[0], 0);
    assert_eq!(t_full_ok(&s, &t), 0);
}

#[test]
fn test_single_a() {
    let s: [usize; 1] = [97]; // "a"
    let t = string_ukkonen::suffix_tree(&s);
    assert_eq!(t.sz, 2);
    assert_eq!(t.next[97], 1); // root --'a'--> node 1
    assert_eq!(t.link[1], 0);
    assert_eq!(t_full_ok(&s, &t), 0);
}

#[test]
fn test_aa_ab() {
    let s1: [usize; 2] = [97, 97]; // "aa"
    let t1 = string_ukkonen::suffix_tree(&s1);
    assert_eq!(t1.sz, 3); // root, "a", "aa"
    assert_eq!(t1.link[1], 0);
    assert_eq!(t1.link[2], 1); // "aa" -> "a"
    assert_eq!(t_full_ok(&s1, &t1), 0);

    let s2: [usize; 2] = [97, 98]; // "ab"
    let t2 = string_ukkonen::suffix_tree(&s2);
    assert_eq!(t2.sz, 4); // root, "a", "ab", "b"
    assert_eq!(t_full_ok(&s2, &t2), 0);
}

#[test]
fn test_aba_abab() {
    let s1: [usize; 3] = [97, 98, 97]; // "aba"
    let t1 = string_ukkonen::suffix_tree(&s1);
    assert_eq!(t1.sz, 6);
    assert_eq!(t_full_ok(&s1, &t1), 0);

    let s2: [usize; 4] = [97, 98, 97, 98]; // "abab"
    let t2 = string_ukkonen::suffix_tree(&s2);
    assert_eq!(t2.sz, 8);
    assert_eq!(t_full_ok(&s2, &t2), 0);
}

#[test]
fn test_banana() {
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97]; // "banana"
    let t = string_ukkonen::suffix_tree(&s);
    // 15 distinct nonempty substrings + the root.
    assert_eq!(t_distinct_count(&s), 15);
    assert_eq!(t.sz, t_distinct_count(&s) + 1);
    assert_eq!(t_single_char_links_ok(&s, &t), 0);
    assert_eq!(t_missed_substrings(&s, &t), 0);

    // "nan" and "nana" occur; "nab" does not.
    let nan: [usize; 3] = [110, 97, 110];
    let nana: [usize; 4] = [110, 97, 110, 97];
    let nab: [usize; 3] = [110, 97, 98];
    assert_eq!(t_word_ok(&t, &nan), 1);
    assert_eq!(t_word_ok(&t, &nana), 1);
    assert_eq!(t_word_ok(&t, &nab), 0);
}

#[test]
fn test_abcabc_aabaa() {
    let s1: [usize; 6] = [97, 98, 99, 97, 98, 99]; // "abcabc"
    let t1 = string_ukkonen::suffix_tree(&s1);
    assert_eq!(t_distinct_count(&s1), 15);
    assert_eq!(t1.sz, t_distinct_count(&s1) + 1);
    assert_eq!(t_missed_substrings(&s1, &t1), 0);
    assert_eq!(t_single_char_links_ok(&s1, &t1), 0);

    let s2: [usize; 5] = [97, 97, 98, 97, 97]; // "aabaa"
    let t2 = string_ukkonen::suffix_tree(&s2);
    assert_eq!(t_distinct_count(&s2), 11);
    assert_eq!(t2.sz, t_distinct_count(&s2) + 1);
    assert_eq!(t_missed_substrings(&s2, &t2), 0);
    assert_eq!(t_single_char_links_ok(&s2, &t2), 0);
}

#[test]
fn test_mississippi() {
    let s: [usize; 11] = [109, 105, 115, 115, 105, 115, 115, 105, 112, 112, 105];
    // "mississippi"
    let t = string_ukkonen::suffix_tree(&s);
    assert_eq!(t.sz, 54); // 53 distinct nonempty substrings + root
    assert_eq!(t_single_char_links_ok(&s, &t), 0);
    // Every suffix (hence every substring) is reachable.
    let r1: [usize; 4] = [105, 115, 115, 105]; // "issi" occurs
    let r2: [usize; 4] = [105, 112, 112, 105]; // "ippi" occurs
    let r3: [usize; 5] = [109, 105, 115, 115, 120]; // "missx" does not
    assert_eq!(t_word_ok(&t, &r1), 1);
    assert_eq!(t_word_ok(&t, &r2), 1);
    assert_eq!(t_word_ok(&t, &r3), 0);
}

#[test]
fn test_suffixes_all_reachable() {
    // Each suffix of "abab" must be reachable from the root.
    let s: [usize; 4] = [97, 98, 97, 98]; // "abab"
    let t = string_ukkonen::suffix_tree(&s);
    let n = s.len();
    let mut missed: usize = 0;
    let mut i = 0;
    while i < n {
        let start = t_walk_buf(&t.next, &s, i, n - i, t.next.len());
        if start == t.next.len() {
            missed += 1;
        }
        i += 1;
    }
    assert_eq!(missed, 0);
}
