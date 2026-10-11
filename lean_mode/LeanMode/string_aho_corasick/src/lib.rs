//! Aho–Corasick multi-pattern matching over `usize`-encoded byte strings.
//!
//! Characters are `usize` byte values (`0..=255`).  `patterns` is the
//! concatenation of every pattern; `offsets` has length `P + 1` and pattern `p`
//! occupies `patterns[offsets[p] .. offsets[p+1]]`.

/// Flat index for the transition table `go[node * alpha + char]`.
fn go_idx(v: usize, alpha: usize, c: usize) -> usize {
    let t = v * alpha;
    let r = t + c;
    r
}

/// Return, for each pattern `p`, whether it occurs somewhere in `text`.
/// An empty pattern always matches.
pub fn aho_corasick(text: &[usize], patterns: &[usize], offsets: &[usize]) -> Vec<bool> {
    let off_len = offsets.len();
    if off_len == 0 {
        return Vec::new();
    }
    let p_count = off_len - 1;
    let total = patterns.len();
    let max_nodes = total + 1;
    let alpha = 256;
    let sentinel = max_nodes;

    let mut matched: Vec<bool> = vec![false; p_count];

    // Empty patterns always match.
    let mut p = 0;
    while p < p_count {
        if offsets[p] == offsets[p + 1] {
            matched[p] = true;
        }
        p += 1;
    }

    let table_len = max_nodes * alpha;
    let mut go: Vec<usize> = vec![sentinel; table_len];
    let mut fail: Vec<usize> = vec![0usize; max_nodes];
    let mut out: Vec<usize> = vec![sentinel; max_nodes];

    // Build the trie.
    let mut nodes = 1;
    let mut p = 0;
    while p < p_count {
        if offsets[p] != offsets[p + 1] {
            let mut v = 0;
            let mut pos = offsets[p];
            let end = offsets[p + 1];
            while pos < end {
                let c = patterns[pos];
                let idx = go_idx(v, alpha, c);
                let child = go[idx];
                if child == sentinel {
                    go[idx] = nodes;
                    v = nodes;
                    nodes += 1;
                } else {
                    v = child;
                }
                pos += 1;
            }
            out[v] = p;
        }
        p += 1;
    }

    // Complete the root's transitions (missing transition → root).
    let mut c = 0;
    while c < alpha {
        let idx = go_idx(0, alpha, c);
        if go[idx] == sentinel {
            go[idx] = 0;
        }
        c += 1;
    }

    // BFS to build failure links.
    let mut queue: Vec<usize> = vec![0usize; max_nodes];
    let mut head = 0;
    let mut tail = 0;
    let mut c = 0;
    while c < alpha {
        let child = go[go_idx(0, alpha, c)];
        if child != 0 {
            queue[tail] = child;
            tail += 1;
        }
        c += 1;
    }
    while head < tail {
        let v = queue[head];
        head += 1;
        let mut c = 0;
        while c < alpha {
            let idx = go_idx(v, alpha, c);
            let u = go[idx];
            if u != sentinel {
                let f = fail[v];
                let fail_idx = go_idx(f, alpha, c);
                let fv = go[fail_idx];
                fail[u] = fv;
                queue[tail] = u;
                tail += 1;
            } else {
                let f = fail[v];
                let fail_idx = go_idx(f, alpha, c);
                let fv = go[fail_idx];
                go[idx] = fv;
            }
            c += 1;
        }
    }

    // Match: walk the text through the goto function, then follow failure
    // links to collect every matched pattern.
    let n = text.len();
    let mut state = 0;
    let mut i = 0;
    while i < n {
        let c = text[i];
        state = go[go_idx(state, alpha, c)];
        let mut u = state;
        let mut more = true;
        while more {
            if out[u] != sentinel {
                matched[out[u]] = true;
            }
            if u == 0 {
                more = false;
            } else {
                u = fail[u];
            }
        }
        i += 1;
    }

    matched
}
