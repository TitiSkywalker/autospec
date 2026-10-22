//! Suffix automaton (SAM) over `usize`-encoded byte strings.
//!
//! Characters are `usize` byte values (`0..=255`).  The SAM is stored as
//! flat arrays: `next[v * alpha + c]` is the transition from state `v` on
//! character `c`, `len[v]` is the length of the longest string in the state's
//! `endpos` equivalence class, and `link[v]` is the suffix link.  State `0` is
//! the initial state (the empty string); `0` also doubles as "no transition"
//! because no transition ever targets the initial state.

const ALPHA: usize = 256;

/// A suffix automaton of a string.
pub struct Sam {
    pub next: Vec<usize>,
    pub len: Vec<usize>,
    pub link: Vec<usize>,
    pub sz: usize,
    pub last: usize,
}

/// Flat index for the transition table `next[v * alpha + c]`.
fn go_idx(v: usize, alpha: usize, c: usize) -> usize {
    let t = v * alpha;
    let r = t + c;
    r
}

/// Build the suffix automaton of `s`.
pub fn sam_build(s: &[usize]) -> Sam {
    let n = s.len();
    let alpha = ALPHA;
    let max_states = 2 * n + 1;
    let table_len = max_states * alpha;
    let mut next: Vec<usize> = vec![0usize; table_len];
    let mut len: Vec<usize> = vec![0usize; max_states];
    let mut link: Vec<usize> = vec![0usize; max_states];

    // State 0 is the root (empty string); `link[0]` is a self-loop used only
    // as a terminator for the suffix-link walks.
    let mut sz = 1;
    let mut last = 0;

    let mut i = 0;
    while i < n {
        let c = s[i];
        let cur = sz;
        sz += 1;
        let llast = len[last];
        let llast1 = llast + 1;
        len[cur] = llast1;

        // Add missing transitions along the suffix-link chain; stop at the
        // first state that already has a transition on `c` (or at the root).
        let mut p = last;
        let mut q = 0;
        let mut done = false;
        while !done {
            let idx = go_idx(p, alpha, c);
            let nv = next[idx];
            if nv == 0 {
                next[idx] = cur;
                if p == 0 {
                    link[cur] = 0;
                    done = true;
                } else {
                    let np = link[p];
                    p = np;
                }
            } else {
                q = nv;
                done = true;
            }
        }

        if q != 0 {
            let lp = len[p];
            let lp1 = lp + 1;
            let lq = len[q];
            if lp1 == lq {
                // Case 2: continuous transition.
                link[cur] = q;
            } else {
                // Case 3: split `q` into a clone state.
                let clone = sz;
                sz += 1;
                len[clone] = lp1;

                // Copy every transition of `q` into `clone`.
                let mut cc = 0;
                while cc < alpha {
                    let qi = go_idx(q, alpha, cc);
                    let t = next[qi];
                    let ci = go_idx(clone, alpha, cc);
                    next[ci] = t;
                    cc += 1;
                }
                let lq_link = link[q];
                link[clone] = lq_link;

                // Redirect transitions pointing to `q` up the suffix links.
                let mut r = p;
                let mut redir_done = false;
                while !redir_done {
                    let idx = go_idx(r, alpha, c);
                    let nv = next[idx];
                    if nv == q {
                        next[idx] = clone;
                        if r == 0 {
                            redir_done = true;
                        } else {
                            let nr = link[r];
                            r = nr;
                        }
                    } else {
                        redir_done = true;
                    }
                }
                link[q] = clone;
                link[cur] = clone;
            }
        }

        last = cur;
        i += 1;
    }

    Sam { next, len, link, sz, last }
}

/// Return whether `p` is a substring of the string the automaton was built
/// for, by walking the transitions from the initial state.
pub fn sam_contains(sam: &Sam, p: &[usize]) -> bool {
    let alpha = ALPHA;
    let n = p.len();
    let mut v = 0;
    let mut i = 0;
    let mut ok = true;
    while i < n && ok {
        let c = p[i];
        let idx = go_idx(v, alpha, c);
        let nv = sam.next[idx];
        if nv == 0 {
            ok = false;
        } else {
            v = nv;
        }
        i += 1;
    }
    ok
}

/// Number of distinct nonempty substrings, via `Σ_v (len[v] - len[link[v]])`.
pub fn sam_distinct_substrings(sam: &Sam) -> usize {
    let sz = sam.sz;
    let mut total = 0;
    let mut v = 1;
    while v < sz {
        let lv = sam.len[v];
        let lk = sam.link[v];
        let ll = sam.len[lk];
        let diff = lv - ll;
        total += diff;
        v += 1;
    }
    total
}
