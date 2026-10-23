//! Cartesian tree.
//!
//! `cartesian_tree(a)` builds the Cartesian tree of the array `a` with the
//! classic O(n) monotonic-stack construction and returns the *parent array*
//! `parent`, where `parent[i]` is the index of the parent of node `i`, or
//! `a.len()` when `i` is the root (the sentinel "no parent").
//!
//! Node `i` carries the value `a[i]`.  The construction produces
//!
//! * a **min-heap**: `a[parent[i]] <= a[i]` for every non-root `i`;
//! * a **binary search tree on the indices**: the in-order traversal visits
//!   `0, 1, ..., n-1` in order.
//!
//! These two properties determine the tree uniquely (when the values are
//! distinct), and they make the tree encode every range-minimum query: the
//! position of the minimum of `a[l..=r]` is the lowest common ancestor of
//! `l` and `r`.
//!
//! The stack is an explicit fixed-capacity `Vec<usize>` with a top counter
//! `sp`, so that a "pop" is just a decrement (no `Vec::pop` / `Option`).

/// Build the Cartesian tree of `a`; returns the parent array.
///
/// `parent[i] == a.len()` means `i` is the root.
pub fn cartesian_tree(a: &[i64]) -> Vec<usize> {
    let n = a.len();
    let mut parent: Vec<usize> = vec![n; n];
    let mut st: Vec<usize> = vec![0usize; n];
    let mut sp: usize = 0;
    let mut i: usize = 0;
    while i < n {
        let sp0 = sp;
        // Pop every stack entry whose value is strictly greater than `a[i]`.
        loop {
            if sp == 0 {
                break;
            }
            let top = st[sp - 1];
            let av = a[top];
            let ai = a[i];
            let pop_it = av > ai;
            if pop_it {
                sp -= 1;
            } else {
                break;
            }
        }
        // The deepest popped node (if any) becomes a child of `i`.  It is the
        // first entry that was popped, i.e. `st[sp]` after the pops.
        if sp < sp0 {
            let last = st[sp];
            parent[last] = i;
        }
        // If the stack is non-empty, its top is the parent of `i`.
        if sp > 0 {
            let top = st[sp - 1];
            parent[i] = top;
        }
        st[sp] = i;
        sp += 1;
        i += 1;
    }
    parent
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- brute-force reference construction (this module is not translated) ----

    /// Fill `parent` with the Cartesian tree of `a[lo..=hi]`, taking the
    /// leftmost minimum of each range as the root (matching the stack
    /// algorithm's tie-breaking).
    fn bf_fill(a: &[i64], parent: &mut Vec<usize>, lo: usize, hi: usize, par: usize) {
        if lo > hi {
            return;
        }
        let mut m = lo;
        let mut best = a[lo];
        let mut k = lo + 1;
        while k <= hi {
            let v = a[k];
            if v < best {
                best = v;
                m = k;
            }
            k += 1;
        }
        parent[m] = par;
        if m > lo {
            bf_fill(a, parent, lo, m - 1, m);
        }
        if m < hi {
            bf_fill(a, parent, m + 1, hi, m);
        }
    }

    fn bf_parent(a: &[i64]) -> Vec<usize> {
        let n = a.len();
        let mut parent: Vec<usize> = vec![n; n];
        if n > 0 {
            bf_fill(a, &mut parent, 0, n - 1, n);
        }
        parent
    }

    fn vec_eq(x: &[usize], y: &[usize]) -> bool {
        if x.len() != y.len() {
            return false;
        }
        let mut i = 0;
        while i < x.len() {
            if x[i] != y[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Min-heap property: `a[parent[i]] <= a[i]` for every non-root `i`.
    fn heap_ok(a: &[i64], parent: &[usize]) -> bool {
        let n = a.len();
        let mut i = 0;
        while i < n {
            let p = parent[i];
            if p < n && a[p] > a[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    /// The (unique) left child of `v` (`n` when absent).  In a BST on the
    /// indices, the left child is the smaller-indexed child.
    fn left_child(parent: &[usize], n: usize, v: usize) -> usize {
        let mut c = n;
        let mut u = 0;
        while u < n {
            if parent[u] == v && u < v {
                c = u;
            }
            u += 1;
        }
        c
    }

    /// The (unique) right child of `v` (`n` when absent).
    fn right_child(parent: &[usize], n: usize, v: usize) -> usize {
        let mut c = n;
        let mut u = 0;
        while u < n {
            if parent[u] == v && u > v {
                c = u;
            }
            u += 1;
        }
        c
    }

    /// Iterative in-order traversal visits `0,1,...,n-1`?
    fn inorder_ok(parent: &[usize], n: usize) -> bool {
        let mut root = n;
        let mut i = 0;
        while i < n {
            if parent[i] == n {
                root = i;
            }
            i += 1;
        }
        if root == n {
            return n == 0;
        }
        let mut cnt: usize = 0;
        let mut stack: Vec<usize> = Vec::new();
        let mut cur = root;
        let mut good = true;
        let mut done = false;
        while !done {
            if cur != n {
                stack.push(cur);
                cur = left_child(parent, n, cur);
            } else if stack.len() > 0 {
                let v = stack[stack.len() - 1];
                stack.pop();
                if v != cnt {
                    good = false;
                }
                cnt += 1;
                cur = right_child(parent, n, v);
            } else {
                done = true;
            }
        }
        good && cnt == n
    }

    /// Value stored at the LCA of `l` and `r` (walking up via a mark array).
    fn lca_val(a: &[i64], parent: &[usize], l: usize, r: usize) -> i64 {
        let n = a.len();
        let mut mark: Vec<u8> = vec![0u8; n + 1];
        let mut x = l;
        mark[x] = 1;
        while parent[x] != n {
            x = parent[x];
            mark[x] = 1;
        }
        let mut y = r;
        while mark[y] == 0 {
            y = parent[y];
        }
        a[y]
    }

    /// The LCA value equals the range minimum for every `l <= r`.
    fn rmq_ok(a: &[i64], parent: &[usize]) -> bool {
        let n = a.len();
        let mut l = 0;
        while l < n {
            let mut r = l;
            while r < n {
                let lv = lca_val(a, parent, l, r);
                let mut mn = a[l];
                let mut k = l;
                while k <= r {
                    if a[k] < mn {
                        mn = a[k];
                    }
                    k += 1;
                }
                if lv != mn {
                    return false;
                }
                r += 1;
            }
            l += 1;
        }
        true
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    #[test]
    fn edge_empty_and_single() {
        let a0: Vec<i64> = Vec::new();
        let p0 = cartesian_tree(&a0);
        assert_eq!(p0.len(), 0);

        let a1 = vec![42i64];
        let p1 = cartesian_tree(&a1);
        assert_eq!(p1.len(), 1);
        assert_eq!(p1[0], 1);
    }

    #[test]
    fn duplicates_use_leftmost_min() {
        let a = vec![2i64, 1, 1, 3, 1];
        let p = cartesian_tree(&a);
        let q = bf_parent(&a);
        assert!(vec_eq(&p, &q));
        assert!(rmq_ok(&a, &p));
        assert_eq!(p[1], 5); // leftmost minimum (index 1) is the root
    }

    #[test]
    fn sorted_and_reverse_sorted() {
        let inc = vec![1i64, 2, 3, 4, 5];
        let p = cartesian_tree(&inc);
        assert!(vec_eq(&p, &bf_parent(&inc)));
        assert!(inorder_ok(&p, inc.len()));
        assert!(heap_ok(&inc, &p));
        // increasing -> a right-leaning chain: parent[i] = i-1, root 0
        assert_eq!(p[0], 5);
        assert_eq!(p[4], 3);

        let dec = vec![5i64, 4, 3, 2, 1];
        let q = cartesian_tree(&dec);
        assert!(vec_eq(&q, &bf_parent(&dec)));
        assert!(inorder_ok(&q, dec.len()));
        assert!(heap_ok(&dec, &q));
        // decreasing -> a left-leaning chain: parent[i] = i+1, root 4
        assert_eq!(q[0], 1);
        assert_eq!(q[4], 5);
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 987654321;
        let mut trial: usize = 0;
        while trial < 500 {
            let r = lcg(&mut state);
            let n = ((r >> 3) % 13) as usize;
            let mut a: Vec<i64> = Vec::new();
            let mut i = 0;
            while i < n {
                let rr = lcg(&mut state);
                // small value range to force many duplicates / ties
                let v = ((rr >> 5) % 7) as i64 - 3;
                a.push(v);
                i += 1;
            }
            let p = cartesian_tree(&a);
            let q = bf_parent(&a);
            assert!(vec_eq(&p, &q), "trial {} mismatch", trial);
            assert!(heap_ok(&a, &p));
            assert!(inorder_ok(&p, a.len()));
            assert!(rmq_ok(&a, &p));
            trial += 1;
        }
    }

    #[test]
    fn randomized_differential_distinct() {
        let mut state: u64 = 13579;
        let mut trial: usize = 0;
        while trial < 300 {
            let r = lcg(&mut state);
            let n = ((r >> 3) % 13) as usize;
            let mut a: Vec<i64> = Vec::new();
            let mut i = 0;
            while i < n {
                let rr = lcg(&mut state);
                // distinct values: use a large multiplier
                let v = ((rr >> 11) % 100000) as i64;
                a.push(v);
                i += 1;
            }
            let p = cartesian_tree(&a);
            let q = bf_parent(&a);
            assert!(vec_eq(&p, &q));
            assert!(heap_ok(&a, &p));
            assert!(inorder_ok(&p, a.len()));
            assert!(rmq_ok(&a, &p));
            trial += 1;
        }
    }
}
