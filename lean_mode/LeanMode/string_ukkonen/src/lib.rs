//! Suffix tree of a `usize`-encoded byte string.
//!
//! Characters are `usize` byte values (`0..=255`).  The tree built here is the
//! *trie of all suffixes* of `s` — the suffix tree with all edges left
//! un-compressed — stored as flat arrays:
//!
//! * `next[v * ALPHA + c]` is the child of node `v` reached by character `c`
//!   (`0` means "no such child").  Node `0` is the root (the empty string) and
//!   is never the target of a transition, so `0` doubles as the "absent"
//!   marker.
//! * `link[v]` is the *suffix link* of node `v`: the node spelling the longest
//!   proper suffix of the string spelled by `v`'s path from the root.  For a
//!   child of the root this is `0` (the empty string); `link[0] = 0`.
//! * `sz` is the number of nodes (root included).
//!
//! A path from the root spells a string that is a prefix of some suffix of `s`,
//! i.e. a substring of `s`; conversely every substring of `s` is a prefix of
//! some suffix.  Hence the non-root nodes are in bijection with the distinct
//! nonempty substrings of `s`, and `sz = 1 + #distinct nonempty substrings`.
//!
//! The construction is the direct `O(n²)` one: insert every suffix `s[i..]`
//! into the trie (walking and creating nodes), then compute the suffix links
//! with a BFS over the trie.  (The `O(n)` Ukkonen construction compresses the
//! edges and maintains an active point; that variant is not representable in
//! the supported Rust subset.)

const ALPHA: usize = 256;

/// A suffix tree (trie of all suffixes) of a string, with suffix links.
pub struct SuffixTree {
    /// Child table: `next[v * ALPHA + c]` (`0` = absent).
    pub next: Vec<usize>,
    /// Suffix link: node of the longest proper suffix of node `v`'s path.
    pub link: Vec<usize>,
    /// Number of nodes (root included).
    pub sz: usize,
}

/// Flat index for the child table `next[v * alpha + c]`.
fn go_idx(v: usize, alpha: usize, c: usize) -> usize {
    let t = v * alpha;
    let r = t + c;
    r
}

/// The distinct characters of `s`, in order of first occurrence.  Every edge of
/// the trie is labelled with a character of `s`, so these are the only
/// characters a node can possibly have a child on.
fn chars_used(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut a: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        let c = s[i];
        let mut seen = false;
        let mut k = 0;
        while k < a.len() {
            let d = a[k];
            if d == c {
                seen = true;
            }
            k += 1;
        }
        if !seen {
            a.push(c);
        }
        i += 1;
    }
    a
}

/// Build the suffix tree of `s`: insert every suffix into a trie, then compute
/// the suffix links with a BFS.
pub fn suffix_tree(s: &[usize]) -> SuffixTree {
    let n = s.len();
    let alpha = ALPHA;

    // Node-count bound: inserting the suffixes of lengths `n, n-1, ..., 1`
    // creates at most `n + (n-1) + ... + 1 = n(n+1)/2` nodes, plus the root.
    let np1 = n + 1;
    let prod = n * np1;
    let half = prod / 2;
    let max_nodes = half + 1;
    let table_len = max_nodes * alpha;

    let mut next: Vec<usize> = vec![0usize; table_len];
    let mut link: Vec<usize> = vec![0usize; max_nodes];

    // Insert every suffix `s[i..]`, creating the nodes it needs.  Node `0` is
    // the root.
    let mut sz: usize = 1;
    let mut i = 0;
    while i < n {
        let mut v = 0;
        let mut j = i;
        while j < n {
            let c = s[j];
            let idx = go_idx(v, alpha, c);
            let nv = next[idx];
            if nv == 0 {
                let nd = sz;
                sz += 1;
                next[idx] = nd;
                v = nd;
            } else {
                v = nv;
            }
            j += 1;
        }
        i += 1;
    }

    // Suffix links, BFS order.  A child of the root has an empty suffix link
    // (`link = 0`); for a node `v` reached from `u` by character `c` the
    // longest proper suffix of `v`'s path is the longest proper suffix of
    // `u`'s path extended by `c`, i.e.
    // `link[v] = next[link[u] * ALPHA + c]` — which already exists because the
    // BFS processes `u` (and hence `link[u]`, of smaller depth) before `v`.
    // The children of a node are scanned over the characters of `s` only (any
    // other character has no edge anywhere), which keeps the scan cheap.
    let chars = chars_used(s);
    let clen = chars.len();
    let mut queue: Vec<usize> = vec![0usize; max_nodes];
    let mut head: usize = 0;
    let mut tail: usize = 0;

    let mut k0 = 0;
    while k0 < clen {
        let c = chars[k0];
        let child = next[c];
        if child != 0 {
            link[child] = 0;
            queue[tail] = child;
            tail += 1;
        }
        k0 += 1;
    }

    while head < tail {
        let u = queue[head];
        head += 1;
        let lu = link[u];
        let mut k = 0;
        while k < clen {
            let c = chars[k];
            let idx = go_idx(u, alpha, c);
            let child = next[idx];
            if child != 0 {
                let lidx = go_idx(lu, alpha, c);
                let lv = next[lidx];
                link[child] = lv;
                queue[tail] = child;
                tail += 1;
            }
            k += 1;
        }
    }

    SuffixTree { next, link, sz }
}
