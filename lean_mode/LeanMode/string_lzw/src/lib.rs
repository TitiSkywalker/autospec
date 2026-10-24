//! LZ78 dictionary compression over `usize`-encoded byte strings.
//!
//! Characters are `usize` byte values (`0..=255`), so the dictionary trie uses a
//! flat `ALPHA = 256` wide child table, exactly like the suffix-tree testcase.
//!
//! The dictionary is a trie whose nodes are the *phrases* of the factorization.
//! Node `0` is the root (the empty phrase, which is never the target of a
//! transition, so `0` doubles as the "absent child" marker).  Node `v >= 1` was
//! created while emitting some factor and stands for a phrase
//! `phrase(v) = phrase(link[v]) · [ch[v]]`:
//!
//! * `next[v * ALPHA + c]` is the child of `v` reached by character `c`
//!   (`0` means "no such child");
//! * `link[v]` is the node of `phrase(v)` with its last character removed (the
//!   *base* of the factor that created `v`), and `link[0] = 0`;
//! * `ch[v]` is the last character of `phrase(v)`;
//! * `sz` is the number of nodes (root included);
//! * `codes` holds, for each factor of the factorization in order, the node of
//!   that factor's word.  The last factor reuses an existing node when the
//!   remaining suffix is exhausted by a phrase (no character follows it), so
//!   `codes` is not necessarily duplicate-free.
//!
//! The greedy step at position `i` reads the longest prefix `s[i..j]` of the
//! remaining suffix that is already a phrase (walking the trie), and then
//! extends it by `s[j]` — creating a fresh node — unless `j = n`, in which case
//! the phrase `s[i..n]` alone is the last factor and no node is created.

const ALPHA: usize = 256;

/// An LZ78 dictionary: the phrase trie plus the factors' node codes.
pub struct Lz78 {
    /// Child table: `next[v * ALPHA + c]` (`0` = absent).
    pub next: Vec<usize>,
    /// Base of the factor that created `v` (parent in the phrase trie).
    pub link: Vec<usize>,
    /// Last character of `phrase(v)`.
    pub ch: Vec<usize>,
    /// Node of each factor's word, in factor order.
    pub codes: Vec<usize>,
    /// Number of nodes (root included).
    pub sz: usize,
}

/// Flat index for the child table `next[v * alpha + c]`.
fn go_idx(v: usize, alpha: usize, c: usize) -> usize {
    let t = v * alpha;
    let r = t + c;
    r
}

/// Build the LZ78 dictionary and factorization of `s`.
pub fn lz78(s: &[usize]) -> Lz78 {
    let n = s.len();
    let alpha = ALPHA;

    // One node per factor at most, plus the root.
    let max_nodes = n + 1;
    let table_len = max_nodes * alpha;

    let mut next: Vec<usize> = vec![0usize; table_len];
    let mut link: Vec<usize> = vec![0usize; max_nodes];
    let mut ch: Vec<usize> = vec![0usize; max_nodes];
    let mut codes: Vec<usize> = Vec::new();

    let mut sz: usize = 1;
    let mut i = 0;
    while i < n {
        // Walk down the trie while the next character keeps us inside the
        // dictionary: `s[i..j]` is the longest phrase starting at `i`.
        let mut v = 0;
        let mut j = i;
        let mut stop = false;
        while !stop {
            if j < n {
                let c = s[j];
                let idx = go_idx(v, alpha, c);
                let nv = next[idx];
                if nv != 0 {
                    v = nv;
                    j += 1;
                } else {
                    stop = true;
                }
            } else {
                stop = true;
            }
        }

        if j == n {
            // The whole remaining suffix is a phrase: the last factor is that
            // phrase itself, and no new node is created.
            codes.push(v);
            i = j;
        } else {
            // Extend the longest phrase by the following character: a fresh
            // node, and the factor is `phrase(v) · [c]`.
            let c = s[j];
            let idx = go_idx(v, alpha, c);
            let nd = sz;
            sz += 1;
            next[idx] = nd;
            link[nd] = v;
            ch[nd] = c;
            codes.push(nd);
            i = j + 1;
        }
    }

    Lz78 { next, link, ch, codes, sz }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference LZ78: the dictionary is a `Vec` of phrase words, the greedy
    /// step takes the longest prefix of the remaining suffix that is a phrase.
    /// Returns the factor words, as `(start, length)` pairs of the input.
    fn reference(s: &[usize]) -> Vec<(usize, usize)> {
        let n = s.len();
        // phrases[p] is stored flat: dict[off[p] .. off[p] + len[p]].
        let mut dict: Vec<usize> = Vec::new();
        let mut off: Vec<usize> = Vec::new();
        let mut len: Vec<usize> = Vec::new();
        let mut factors: Vec<(usize, usize)> = Vec::new();
        off.push(0);
        len.push(0); // the empty phrase is node 0
        let mut i = 0;
        while i < n {
            // Longest phrase that is a prefix of s[i..].
            let mut best = 0;
            let mut p = 0;
            while p < off.len() {
                let l = len[p];
                let mut ok = true;
                if i + l > n {
                    ok = false;
                } else {
                    let mut k = 0;
                    while k < l {
                        if dict[off[p] + k] != s[i + k] {
                            ok = false;
                        }
                        k += 1;
                    }
                }
                if ok && l > len[best] {
                    best = p;
                }
                p += 1;
            }
            let l = len[best];
            // The factor is the longest phrase plus one character, or — when
            // the phrase already exhausts the remaining suffix — the phrase
            // itself (so a trailing factor can have no extension).
            let flen = if i + l < n { l + 1 } else { l };
            factors.push((i, flen));
            if i + l == n {
                // Trailing factor: the phrase itself, no new node.
                i = n;
            } else {
                // New node: the phrase plus one character.
                let start = off[off.len() - 1] + len[off.len() - 1];
                let mut k = 0;
                while k < l {
                    dict.push(dict[off[best] + k]);
                    k += 1;
                }
                dict.push(s[i + l]);
                off.push(start);
                len.push(l + 1);
                i = i + l + 1;
            }
        }
        factors
    }

    /// The phrase of node `v` of `t`, recovered by walking the parent links.
    fn phrase(t: &Lz78, v: usize) -> Vec<usize> {
        let mut rev: Vec<usize> = Vec::new();
        let mut cur = v;
        while cur != 0 {
            rev.push(t.ch[cur]);
            cur = t.link[cur];
        }
        let m = rev.len();
        let mut out: Vec<usize> = Vec::new();
        let mut k = 0;
        while k < m {
            let idx = m - 1 - k;
            out.push(rev[idx]);
            k += 1;
        }
        out
    }

    /// The factor words, decoded from the codes.
    fn factor_words(t: &Lz78) -> Vec<Vec<usize>> {
        let mut fs: Vec<Vec<usize>> = Vec::new();
        let mut k = 0;
        while k < t.codes.len() {
            fs.push(phrase(t, t.codes[k]));
            k += 1;
        }
        fs
    }

    fn check(s: &[usize]) {
        let t = lz78(s);
        let fs = factor_words(&t);
        // 1. the factors tile s
        let mut acc: Vec<usize> = Vec::new();
        let mut k = 0;
        while k < fs.len() {
            let mut j = 0;
            while j < fs[k].len() {
                acc.push(fs[k][j]);
                j += 1;
            }
            k += 1;
        }
        assert_eq!(acc.len(), s.len());
        let mut k = 0;
        while k < s.len() {
            assert_eq!(acc[k], s[k]);
            k += 1;
        }
        // 2. every factor word is a substring of s and occurs as a prefix of the
        //    remaining input at the factor's start position
        let mut pos = 0;
        let mut k = 0;
        while k < fs.len() {
            let f = &fs[k];
            let mut j = 0;
            while j < f.len() {
                assert_eq!(f[j], s[pos + j]);
                j += 1;
            }
            pos = pos + f.len();
            k += 1;
        }
        assert_eq!(pos, s.len());
        // 3. the trie recognizes exactly the factor words (and the empty word)
        let mut k = 0;
        while k < t.codes.len() {
            assert_eq!(node_of(&t, &fs[k]), t.codes[k]);
            k += 1;
        }
        // 4. node count = distinct factor words + 1
        let mut distinct: usize = 0;
        let mut k = 0;
        while k < fs.len() {
            let mut first = true;
            let mut m = 0;
            while m < k {
                if same_vec(&fs[m], &fs[k]) {
                    first = false;
                }
                m += 1;
            }
            if first {
                distinct += 1;
            }
            k += 1;
        }
        assert_eq!(t.sz, distinct + 1);
        // 5. the greedy choice: at each factor's start position, no *earlier*
        //    factor is a longer prefix of the remaining input than the factor's
        //    base (the factor without its last character; the whole factor when
        //    the input is exhausted)
        let mut pos = 0;
        let mut k = 0;
        while k < fs.len() {
            let f = &fs[k];
            assert!(f.len() >= 1);
            let rem_len = s.len() - pos;
            let base_len = if f.len() < rem_len { f.len() - 1 } else { f.len() };
            let mut j = 0;
            while j < k {
                let g = &fs[j];
                let mut is_pref = g.len() <= rem_len;
                let mut m = 0;
                while m < g.len() && m < rem_len {
                    if g[m] != s[pos + m] {
                        is_pref = false;
                    }
                    m += 1;
                }
                if is_pref {
                    assert!(g.len() <= base_len);
                }
                j += 1;
            }
            pos = pos + f.len();
            k += 1;
        }
        assert_eq!(pos, s.len());
        // 6. cross-check the factorization against the brute-force reference
        let want = reference(s);
        assert_eq!(fs.len(), want.len());
        let mut pos = 0;
        let mut k = 0;
        while k < fs.len() {
            assert_eq!(fs[k].len(), want[k].1);
            assert_eq!(pos, want[k].0);
            pos = pos + fs[k].len();
            k += 1;
        }
    }

    /// Element-wise equality of two `Vec<usize>` (`Vec` has no `PartialEq` in
    /// the translated subset, so spell it out).
    fn same_vec(a: &[usize], b: &[usize]) -> bool {
        let mut same = a.len() == b.len();
        let mut k = 0;
        while k < a.len() && k < b.len() {
            if a[k] != b[k] {
                same = false;
            }
            k += 1;
        }
        same
    }

    /// Walk `w` down the child table; returns the node it ends in, or `nf`.
    fn node_of(t: &Lz78, w: &[usize]) -> usize {
        let nf = t.next.len();
        let mut v = 0;
        let mut i = 0;
        let mut ok = true;
        while i < w.len() && ok {
            let c = w[i];
            let idx = v * 256 + c;
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

    #[test]
    fn test_small_fixed() {
        check(&[]);
        check(&[97]);
        check(&[97, 97]);
        check(&[97, 98]);
        check(&[97, 97, 97]);
        check(&[97, 98, 99, 97, 98]);
        check(&[97, 98, 97, 98, 97, 98]);
    }

    #[test]
    fn test_lz78_known_factorizations() {
        // "aaa": factors "a", "aa"
        let t = lz78(&[97, 97, 97]);
        assert_eq!(t.codes.len(), 2);
        assert_eq!(t.sz, 3);
        // "abcab": factors "a", "b", "c", "ab"
        let t2 = lz78(&[97, 98, 99, 97, 98]);
        assert_eq!(t2.codes.len(), 4);
        assert_eq!(t2.sz, 5);
    }

    #[test]
    fn test_fuzz_exhaustive_small() {
        // all strings over {0,1} up to length 9, and over {0,1,2} up to length 6
        let mut n = 0;
        while n <= 9 {
            let mut code = 0;
            let mut total: u64 = 1;
            let mut k = 0;
            while k < n {
                total = total * 2;
                k += 1;
            }
            while code < total {
                let mut buf: Vec<usize> = Vec::new();
                let mut k = 0;
                let mut c = code;
                while k < n {
                    buf.push((c % 2) as usize);
                    c = c / 2;
                    k += 1;
                }
                check(&buf);
                code += 1;
            }
            n += 1;
        }
        let mut n = 0;
        while n <= 6 {
            let mut code = 0;
            let mut total: u64 = 1;
            let mut k = 0;
            while k < n {
                total = total * 3;
                k += 1;
            }
            while code < total {
                let mut buf: Vec<usize> = Vec::new();
                let mut k = 0;
                let mut c = code;
                while k < n {
                    buf.push((c % 3) as usize);
                    c = c / 3;
                    k += 1;
                }
                check(&buf);
                code += 1;
            }
            n += 1;
        }
    }

    #[test]
    fn test_fibonacci_and_powers() {
        // Fibonacci word, and a few highly repetitive strings
        let mut a: Vec<usize> = vec![0];
        let mut b: Vec<usize> = vec![0, 1];
        let mut k = 0;
        while k < 10 {
            let mut c = a.clone();
            let mut j = 0;
            while j < b.len() {
                c.push(b[j]);
                j += 1;
            }
            a = b;
            b = c;
            k += 1;
        }
        check(&a);
        check(&b);

        let mut s: Vec<usize> = Vec::new();
        let mut k = 0;
        while k < 40 {
            s.push(k % 3);
            k += 1;
        }
        check(&s);
    }
}
