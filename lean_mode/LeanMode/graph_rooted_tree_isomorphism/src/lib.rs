//! Rooted tree isomorphism via the AHU algorithm (canonical-form induction).

/// Insertion-sort a small `usize` vector in place (used to canonicalise the
/// multiset of a node's child labels).
fn sort_vec(mut v: Vec<usize>) -> Vec<usize> {
    let n = v.len();
    let mut i: usize = 1;
    while i < n {
        let key = v[i];
        let mut j: usize = i;
        let mut moved: bool = true;
        while j > 0 && moved {
            let prev = v[j - 1];
            if prev > key {
                v[j] = prev;
                j -= 1;
            } else {
                moved = false;
            }
        }
        v[j] = key;
        i += 1;
    }
    v
}

/// Compare the *sorted* child-label lists (signatures) of two nodes
/// lexicographically.  Returns `0` when `u`'s signature is smaller, `1` when
/// they are equal, `2` when `u`'s is larger.
fn sig_cmp(sig: &[usize], off: &[usize], len: &[usize], u: usize, w: usize) -> usize {
    let lu = len[u];
    let lw = len[w];
    let mut lim = lu;
    if lw < lim {
        lim = lw;
    }
    let mut i: usize = 0;
    let mut result: usize = 1;
    let mut decided: bool = false;
    while i < lim && !decided {
        let ou = off[u];
        let ow = off[w];
        let iu = ou + i;
        let iw = ow + i;
        let au = sig[iu];
        let aw = sig[iw];
        if au < aw {
            result = 0;
            decided = true;
        } else {
            if aw < au {
                result = 2;
                decided = true;
            }
        }
        i += 1;
    }
    if decided {
        result
    } else {
        if lu < lw {
            0
        } else {
            if lw < lu {
                2
            } else {
                1
            }
        }
    }
}

/// Decide whether two rooted trees are isomorphic (AHU algorithm).
///
/// Each tree is given by a parent array: `parent[v]` is the parent of `v`, and
/// the root satisfies `parent[root] = root` (the root's entry is ignored).
/// `parent1` has length `n1`, `parent2` length `n2`; `root1 < n1`, `root2 < n2`.
///
/// The two trees are processed *jointly*, level by level from the deepest up:
/// a node's signature is the sorted multiset of its children's (already
/// assigned) labels, and within each level distinct signatures are re-ranked to
/// consecutive integers.  This canonical form is a complete invariant of rooted
/// tree isomorphism, so the roots are isomorphic iff they end up with the same
/// label.  The subtle part of the proof is the induction that re-ranking
/// preserves equality of the canonical forms.
pub fn rooted_tree_isomorphism(
    n1: usize, parent1: &[usize], root1: usize,
    n2: usize, parent2: &[usize], root2: usize,
) -> bool {
    let n = n1 + n2;

    // Combined parent array: tree-1 nodes are `0 .. n1`, tree-2 nodes are
    // `n1 .. n1 + n2`.
    let mut par: Vec<usize> = vec![0usize; n];
    let mut i: usize = 0;
    while i < n1 {
        par[i] = parent1[i];
        i += 1;
    }
    while i < n {
        let j = i - n1;
        let p = parent2[j];
        let np = n1 + p;
        par[i] = np;
        i += 1;
    }
    let r1 = root1;
    let r2 = n1 + root2;
    par[r1] = r1;
    par[r2] = r2;

    // Depth of every vertex (the two roots have depth 0).
    let mut depth: Vec<usize> = vec![0usize; n];
    let mut pass: usize = 0;
    while pass < n {
        let mut v: usize = 0;
        while v < n {
            let p = par[v];
            if p != v {
                let d = depth[p] + 1;
                depth[v] = d;
            }
            v += 1;
        }
        pass += 1;
    }

    // Maximum depth.
    let mut maxd: usize = 0;
    let mut v2: usize = 0;
    while v2 < n {
        let d = depth[v2];
        if d > maxd {
            maxd = d;
        }
        v2 += 1;
    }

    // Children lists in CSR form.
    let mut deg: Vec<usize> = vec![0usize; n];
    let mut v3: usize = 0;
    while v3 < n {
        let p = par[v3];
        if p != v3 {
            let d = deg[p] + 1;
            deg[p] = d;
        }
        v3 += 1;
    }
    let mut start: Vec<usize> = vec![0usize; n + 1];
    let mut u: usize = 0;
    while u < n {
        let s = start[u] + deg[u];
        start[u + 1] = s;
        u += 1;
    }
    let total_edges = start[n];
    let mut child: Vec<usize> = vec![0usize; total_edges];
    let mut pos: Vec<usize> = vec![0usize; n];
    let mut u2: usize = 0;
    while u2 < n {
        pos[u2] = start[u2];
        u2 += 1;
    }
    let mut v4: usize = 0;
    while v4 < n {
        let p = par[v4];
        if p != v4 {
            let c = pos[p];
            child[c] = v4;
            let nc = c + 1;
            pos[p] = nc;
        }
        v4 += 1;
    }

    // AHU labels; leaves (no children) start at 0.
    let mut label: Vec<usize> = vec![0usize; n];

    let mut cur: usize = maxd;
    while cur > 0 {
        let d = cur - 1;

        // Collect all nodes at depth `d` (both trees together).
        let mut lev: Vec<usize> = Vec::new();
        let mut v5: usize = 0;
        while v5 < n {
            if depth[v5] == d {
                lev.push(v5);
            }
            v5 += 1;
        }
        let lv = lev.len();

        // Build each node's signature = sorted list of its children's labels,
        // stored flatly (indexed by node id).
        let mut sig: Vec<usize> = Vec::new();
        let mut sig_off: Vec<usize> = vec![0usize; n];
        let mut sig_len: Vec<usize> = vec![0usize; n];
        let mut idx2: usize = 0;
        while idx2 < lv {
            let uu = lev[idx2];
            let sl = sig.len();
            sig_off[uu] = sl;
            let du = deg[uu];
            sig_len[uu] = du;
            let mut tmp: Vec<usize> = Vec::new();
            let mut c: usize = 0;
            while c < du {
                let base = start[uu];
                let ci = base + c;
                let ch = child[ci];
                let lb = label[ch];
                tmp.push(lb);
                c += 1;
            }
            let tmp2 = sort_vec(tmp);
            let mut c2: usize = 0;
            while c2 < du {
                let t2 = tmp2[c2];
                sig.push(t2);
                c2 += 1;
            }
            idx2 += 1;
        }

        // Insertion-sort `lev` by signature.
        let mut a: usize = 1;
        while a < lv {
            let key_node = lev[a];
            let mut b: usize = a;
            let mut moved: bool = true;
            while b > 0 && moved {
                let prev_node = lev[b - 1];
                let cmp = sig_cmp(&sig, &sig_off, &sig_len, prev_node, key_node);
                if cmp == 2 {
                    lev[b] = prev_node;
                    b -= 1;
                } else {
                    moved = false;
                }
            }
            lev[b] = key_node;
            a += 1;
        }

        // Re-rank: equal signatures share a rank.
        let mut rank: usize = 0;
        let mut a2: usize = 0;
        while a2 < lv {
            let uu = lev[a2];
            if a2 > 0 {
                let prev_u = lev[a2 - 1];
                let cmp = sig_cmp(&sig, &sig_off, &sig_len, prev_u, uu);
                if cmp != 1 {
                    let nr = rank + 1;
                    rank = nr;
                }
            }
            label[uu] = rank;
            a2 += 1;
        }

        cur -= 1;
    }

    // Isomorphic iff the two roots share an AHU label.
    let l1 = label[r1];
    let l2 = label[r2];
    if l1 != l2 {
        false
    } else {
        true
    }
}
