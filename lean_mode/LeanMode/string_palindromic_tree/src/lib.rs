//! Palindromic tree (eertree) over `usize`-encoded byte strings.
//!
//! Characters are `usize` byte values (`0..=255`).  The eertree of a string `s`
//! has one node per distinct palindromic substring of `s`, plus two roots:
//!
//! * node `0` is the *odd root*, standing for the imaginary palindrome of
//!   length `-1`;
//! * node `1` is the *even root*, standing for the empty palindrome of length
//!   `0`;
//! * nodes `v >= 2` are the distinct palindromic substrings of `s`, created in
//!   order of first occurrence (so node `2` is the palindrome `s[i]..s[i]` that
//!   appeared first, etc.).
//!
//! To stay free of signed integers the arrays store the **shifted** length
//! `L(v) = actual_len(v) + 1`: `L(0) = 0`, `L(1) = 1`, and a node created by
//! extending node `cur` with character `c` has `L(new) = L(cur) + 2`.  Hence
//! the real palindrome length of node `v >= 2` is `L(v) - 1`.
//!
//! The tree is stored as flat arrays: `next[v * ALPHA + c]` is the child node
//! for the palindrome `c · pal(v) · c` (`0` means "no such child"), `link[v]`
//! is the node of the longest *proper* palindromic suffix of `pal(v)`, and `sz`
//! is the number of nodes.  `link[0] = 0` and `link[1] = 0`; the self-loop on
//! the odd root terminates every suffix-link walk.

const ALPHA: usize = 256;

/// An eertree (palindromic tree) of a string.
pub struct Eertree {
    /// Shifted palindrome length `L(v) = actual_len(v) + 1`.
    pub len: Vec<usize>,
    /// Node of the longest proper palindromic suffix of node `v`.
    pub link: Vec<usize>,
    /// Child table: `next[v * ALPHA + c]` (`0` = absent).
    pub next: Vec<usize>,
    /// Number of nodes (roots included).
    pub sz: usize,
}

/// Flat index for the child table `next[v * alpha + c]`.
fn go_idx(v: usize, alpha: usize, c: usize) -> usize {
    let t = v * alpha;
    let r = t + c;
    r
}

/// Build the eertree of `s` (amortized-linear palindromic tree construction).
pub fn eertree_build(s: &[usize]) -> Eertree {
    let n = s.len();
    let alpha = ALPHA;
    let max_nodes = n + 2;
    let table_len = max_nodes * alpha;
    let mut next: Vec<usize> = vec![0usize; table_len];
    let mut len: Vec<usize> = vec![0usize; max_nodes];
    let mut link: Vec<usize> = vec![0usize; max_nodes];

    // Node 0: odd root, shifted length 0.  Node 1: even root, shifted length 1.
    len[1] = 1;
    let mut sz = 2;
    let mut last = 1;

    let mut i = 0;
    while i < n {
        let c = s[i];

        // Walk the suffix links from `last` down to the largest suffix
        // palindrome that can be extended by `c`.  A node `v` is extendable at
        // position `i` iff `i >= L(v)` and `s[i - L(v)] == c`; for the odd root
        // (`L = 0`) this is `s[i] == c`, which always holds.
        let mut cur = last;
        let mut done = false;
        while !done {
            let lc = len[cur];
            if i >= lc {
                let j = i - lc;
                let sj = s[j];
                if sj == c {
                    done = true;
                } else {
                    let nc = link[cur];
                    cur = nc;
                }
            } else {
                let nc = link[cur];
                cur = nc;
            }
        }

        let idx = go_idx(cur, alpha, c);
        let nx = next[idx];
        if nx != 0 {
            // The palindrome `c · pal(cur) · c` already exists.
            last = nx;
        } else {
            // Create the node for `c · pal(cur) · c`.
            let nd = sz;
            sz += 1;
            let lcur = len[cur];
            let lnew = lcur + 2;
            len[nd] = lnew;
            if lnew == 2 {
                // A single character: its longest proper palindromic suffix is
                // the empty palindrome, i.e. the even root.
                link[nd] = 1;
            } else {
                // Walk from `link[cur]` to the largest suffix palindrome of the
                // new node extendable by `c`; its child on `c` is `link[nd]`.
                let mut p = link[cur];
                let mut done2 = false;
                while !done2 {
                    let lp = len[p];
                    if i >= lp {
                        let j2 = i - lp;
                        let sj2 = s[j2];
                        if sj2 == c {
                            done2 = true;
                        } else {
                            let np = link[p];
                            p = np;
                        }
                    } else {
                        let np = link[p];
                        p = np;
                    }
                }
                let pidx = go_idx(p, alpha, c);
                let pnx = next[pidx];
                link[nd] = pnx;
            }
            next[idx] = nd;
            last = nd;
        }
        i += 1;
    }

    Eertree { len, link, next, sz }
}
