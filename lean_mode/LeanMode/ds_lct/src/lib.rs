//! Link-Cut Tree (array-based, 1-indexed; node `0` is the null sentinel).
//!
//! Maintains a dynamic forest with `link` / `cut` / `findroot` / `path_sum`.
//! The core `access` operation restructures the preferred-path decomposition
//! so that the path from a node to its tree root lives in one splay; `makeroot`
//! reverses a path to re-root a tree.  `path_sum(x, y)` returns the sum of the
//! node values on the `x`–`y` path.

pub struct Lct {
    pub fa: Vec<usize>,
    pub ch0: Vec<usize>,
    pub ch1: Vec<usize>,
    pub rev: Vec<usize>,
    pub val: Vec<i64>,
    pub sum: Vec<i64>,
    pub n: usize,
}

/// 1 if `x` is the right child of its splay parent, else 0.
fn get(st: &Lct, x: usize) -> usize {
    let f = st.fa[x];
    let c1 = st.ch1[f];
    if c1 == x { 1 } else { 0 }
}

/// Is `x` a splay root?  (Its parent does not point back at it.)
fn isroot(st: &Lct, x: usize) -> bool {
    let f = st.fa[x];
    let c0 = st.ch0[f];
    let c1 = st.ch1[f];
    if c0 == x {
        false
    } else if c1 == x {
        false
    } else {
        true
    }
}

/// Toggle a 0/1 flag.
fn toggle(b: usize) -> usize {
    if b == 0 { 1 } else { 0 }
}

/// Recompute `sum[x]` from its two children and own value.
fn pushup(st: Lct, x: usize) -> Lct {
    let mut s = st;
    let l = s.ch0[x];
    let r = s.ch1[x];
    let sl = s.sum[l];
    let sr = s.sum[r];
    let v = s.val[x];
    let t = sl + sr + v;
    s.sum[x] = t;
    s
}

/// Push `x`'s reverse tag down to its children.
fn pushdown(st: Lct, x: usize) -> Lct {
    let mut s = st;
    let rv = s.rev[x];
    if rv == 0 {
        s
    } else {
        let c0 = s.ch0[x];
        let c1 = s.ch1[x];
        s.ch0[x] = c1;
        s.ch1[x] = c0;
        let rl = s.rev[c0];
        let rl2 = toggle(rl);
        s.rev[c0] = rl2;
        let rr = s.rev[c1];
        let rr2 = toggle(rr);
        s.rev[c1] = rr2;
        let z = 0;
        s.rev[x] = z;
        s
    }
}

/// Splay rotation: rotate `x` up one level.
fn rotate(st: Lct, x: usize) -> Lct {
    let mut s = st;
    let y = s.fa[x];
    let z = s.fa[y];
    let k = get(&s, x);
    if !isroot(&s, y) {
        let yk = get(&s, y);
        if yk == 0 {
            s.ch0[z] = x;
        } else {
            s.ch1[z] = x;
        }
    }
    let nk = toggle(k);
    let b = if nk == 0 { s.ch0[x] } else { s.ch1[x] };
    if k == 0 {
        s.ch0[y] = b;
    } else {
        s.ch1[y] = b;
    }
    s.fa[b] = y;
    if nk == 0 {
        s.ch0[x] = y;
    } else {
        s.ch1[x] = y;
    }
    s.fa[y] = x;
    s.fa[x] = z;
    let s = pushup(s, y);
    let s = pushup(s, x);
    s
}

/// Push down reverse tags along the path from the splay root to `x`.
fn update(st: Lct, x: usize) -> Lct {
    let mut s = st;
    if !isroot(&s, x) {
        let f = s.fa[x];
        s = update(s, f);
    }
    let s = pushdown(s, x);
    s
}

/// Splay `x` to the root of its splay tree.
fn splay(st: Lct, x: usize) -> Lct {
    let mut s = st;
    s = update(s, x);
    while !isroot(&s, x) {
        let fa = s.fa[x];
        if !isroot(&s, fa) {
            let gx = get(&s, x);
            let gf = get(&s, fa);
            if gx == gf {
                s = rotate(s, fa);
            } else {
                s = rotate(s, x);
            }
        }
        s = rotate(s, x);
    }
    s
}

/// Make the root-to-`x` path a single preferred path (in one splay).  Returns
/// the updated struct and the last virtual-edge parent (the old splay root).
fn access(st: Lct, x: usize) -> (Lct, usize) {
    let mut s = st;
    let mut p = 0;
    let mut cur = x;
    while cur != 0 {
        s = splay(s, cur);
        s.ch1[cur] = p;
        s = pushup(s, cur);
        let f = s.fa[cur];
        p = cur;
        cur = f;
    }
    (s, p)
}

/// Make `x` the root of its tree.
fn makeroot(st: Lct, x: usize) -> Lct {
    let mut s = st;
    let (s1, _) = access(s, x);
    s = s1;
    s = splay(s, x);
    let rv = s.rev[x];
    let rv2 = toggle(rv);
    s.rev[x] = rv2;
    s
}

/// Build an LCT over nodes `1 .. n` with the given values.
pub fn lct_build(n: usize, vals: &[i64]) -> Lct {
    let size = n + 1;
    let mut fa: Vec<usize> = vec![0; size];
    let mut ch0: Vec<usize> = vec![0; size];
    let mut ch1: Vec<usize> = vec![0; size];
    let mut rev: Vec<usize> = vec![0; size];
    let mut val: Vec<i64> = vec![0; size];
    let mut sum: Vec<i64> = vec![0; size];
    let mut i: usize = 1;
    while i <= n {
        let v = vals[i - 1];
        val[i] = v;
        sum[i] = v;
        fa[i] = 0;
        ch0[i] = 0;
        ch1[i] = 0;
        rev[i] = 0;
        i += 1;
    }
    Lct { fa, ch0, ch1, rev, val, sum, n }
}

/// Connect `x` and `y` (they must currently be in different trees).
pub fn lct_link(st: Lct, x: usize, y: usize) -> Lct {
    let mut s = st;
    s = makeroot(s, x);
    s.fa[x] = y;
    s
}

/// Cut the edge `(x, y)` (it must currently exist).
pub fn lct_cut(st: Lct, x: usize, y: usize) -> Lct {
    let mut s = st;
    s = makeroot(s, x);
    let (s1, _) = access(s, y);
    s = s1;
    s = splay(s, y);
    let z = 0;
    s.ch0[y] = z;
    s.fa[x] = z;
    s
}

/// Root of `x`'s tree.
pub fn lct_findroot(st: Lct, x: usize) -> (Lct, usize) {
    let mut s = st;
    let (s1, _) = access(s, x);
    s = s1;
    s = splay(s, x);
    let mut cur = x;
    while s.ch0[cur] != 0 {
        let l = s.ch0[cur];
        cur = l;
        let s2 = pushdown(s, cur);
        s = s2;
    }
    s = splay(s, cur);
    (s, cur)
}

/// Sum of node values on the `x`–`y` path.
pub fn lct_path_sum(st: Lct, x: usize, y: usize) -> (Lct, i64) {
    let mut s = st;
    s = makeroot(s, x);
    let (s, _) = access(s, y);
    let s = splay(s, y);
    let sm = s.sum[y];
    (s, sm)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference forest as an undirected adjacency list.
    fn path_sum_ref(adj: &Vec<Vec<usize>>, vals: &Vec<i64>, x: usize, y: usize) -> i64 {
        // DFS to find the x-y path.
        let n = adj.len();
        let mut parent: Vec<usize> = vec![0; n];
        let mut visited: Vec<usize> = vec![0; n];
        let mut stack: Vec<usize> = Vec::new();
        stack.push(x);
        visited[x] = 1;
        parent[x] = 0;
        while stack.len() > 0 {
            let u = stack[stack.len() - 1];
            stack.pop();
            if u == y {
                break;
            }
            let mut i: usize = 0;
            while i < adj[u].len() {
                let v = adj[u][i];
                if visited[v] == 0 {
                    visited[v] = 1;
                    parent[v] = u;
                    stack.push(v);
                }
                i += 1;
            }
        }
        let mut s = vals[y - 1];
        let mut cur = y;
        while cur != x {
            cur = parent[cur];
            s = s + vals[cur - 1];
        }
        s
    }

    fn connected(adj: &Vec<Vec<usize>>, x: usize, y: usize) -> bool {
        let n = adj.len();
        let mut visited: Vec<usize> = vec![0; n];
        let mut stack: Vec<usize> = Vec::new();
        stack.push(x);
        visited[x] = 1;
        while stack.len() > 0 {
            let u = stack[stack.len() - 1];
            stack.pop();
            if u == y {
                return true;
            }
            let mut i: usize = 0;
            while i < adj[u].len() {
                let v = adj[u][i];
                if visited[v] == 0 {
                    visited[v] = 1;
                    stack.push(v);
                }
                i += 1;
            }
        }
        false
    }

    fn has_edge(adj: &Vec<Vec<usize>>, x: usize, y: usize) -> bool {
        let mut i: usize = 0;
        while i < adj[x].len() {
            if adj[x][i] == y {
                return true;
            }
            i += 1;
        }
        false
    }

    fn add_edge(adj: &mut Vec<Vec<usize>>, x: usize, y: usize) {
        adj[x].push(y);
        adj[y].push(x);
    }

    fn remove_edge(adj: &mut Vec<Vec<usize>>, x: usize, y: usize) {
        let mut i: usize = 0;
        while i < adj[x].len() {
            if adj[x][i] == y {
                adj[x].remove(i);
                break;
            }
            i += 1;
        }
        let mut i: usize = 0;
        while i < adj[y].len() {
            if adj[y][i] == x {
                adj[y].remove(i);
                break;
            }
            i += 1;
        }
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *state
    }

    #[test]
    fn basic_path_sum() {
        let n = 5;
        let vals = vec![1, 2, 3, 4, 5];
        let mut t = lct_build(n, &vals);
        // link 1-2, 2-3, 3-4
        t = lct_link(t, 1, 2);
        t = lct_link(t, 2, 3);
        t = lct_link(t, 3, 4);
        // path 1..4 = 1+2+3+4 = 10
        let (t, s) = lct_path_sum(t, 1, 4);
        assert_eq!(s, 10);
        // path 2..4 = 2+3+4 = 9
        let (_t, s2) = lct_path_sum(t, 2, 4);
        assert_eq!(s2, 9);
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 2024;
        let n: usize = 30;
        let mut vals: Vec<i64> = Vec::new();
        let mut i: usize = 0;
        while i < n {
            let r = lcg(&mut state);
            vals.push(((r >> 4) % 100) as i64);
            i += 1;
        }
        let mut t = lct_build(n, &vals);
        let mut adj: Vec<Vec<usize>> = Vec::new();
        let mut i: usize = 0;
        while i <= n {
            adj.push(Vec::new());
            i += 1;
        }
        let mut ops: usize = 0;
        while ops < 3000 {
            let r = lcg(&mut state);
            let x = ((r >> 4) % n as u64) as usize + 1;
            let y = ((r >> 12) % n as u64) as usize + 1;
            if x == y {
                ops += 1;
                continue;
            }
            let op = (r >> 20) % 3;
            if op == 0 {
                // link
                if !connected(&adj, x, y) {
                    t = lct_link(t, x, y);
                    add_edge(&mut adj, x, y);
                }
            } else if op == 1 {
                // cut
                if has_edge(&adj, x, y) {
                    t = lct_cut(t, x, y);
                    remove_edge(&mut adj, x, y);
                }
            } else {
                // path sum (only if connected)
                if connected(&adj, x, y) {
                    let (t2, s) = lct_path_sum(t, x, y);
                    t = t2;
                    assert_eq!(s, path_sum_ref(&adj, &vals, x, y));
                }
            }
            ops += 1;
        }
    }
}
