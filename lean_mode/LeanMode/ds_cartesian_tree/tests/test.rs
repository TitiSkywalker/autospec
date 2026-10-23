use ds_cartesian_tree::cartesian_tree;

/// `parent[i]`, or the sentinel `n` when `i` is out of range.
fn t_par(p: &[usize], i: usize, n: usize) -> usize {
    if i < n {
        p[i]
    } else {
        n
    }
}

/// Leftmost index of the minimum of `a[lo..=hi]`.
fn t_argmin(a: &[i64], lo: usize, hi: usize) -> usize {
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
    m
}

/// Min-heap property of the parent array?
fn t_heap_ok(a: &[i64], p: &[usize], n: usize) -> bool {
    let mut ok = true;
    let mut i = 0;
    while i < n {
        let par = t_par(p, i, n);
        if par < n {
            if a[par] > a[i] {
                ok = false;
            }
        }
        i += 1;
    }
    ok
}

/// Value stored at the lowest common ancestor of `l` and `r`.
fn t_lca_val(a: &[i64], p: &[usize], n: usize, l: usize, r: usize) -> i64 {
    let sz = n + 1;
    let mut mark: Vec<usize> = vec![0usize; sz];
    let mut x = l;
    mark[x] = 1;
    while t_par(p, x, n) != n {
        x = t_par(p, x, n);
        mark[x] = 1;
    }
    let mut y = r;
    while mark[y] == 0 {
        y = t_par(p, y, n);
    }
    a[y]
}

#[test]
fn test_parent_array() {
    let a = vec![3i64, 1, 4, 1, 5, 9, 2, 6];
    let p = cartesian_tree(&a);
    assert_eq!(p.len(), 8);
    assert_eq!(p[0], 1);
    assert_eq!(p[1], 8);
    assert_eq!(p[2], 3);
    assert_eq!(p[3], 1);
    assert_eq!(p[4], 6);
    assert_eq!(p[5], 4);
    assert_eq!(p[6], 3);
    assert_eq!(p[7], 6);
}

#[test]
fn test_heap_and_rmq() {
    let a = vec![4i64, 2, 7, 2, 9, 1, 5, 3, 8];
    let p = cartesian_tree(&a);
    let n = a.len();
    let okh = t_heap_ok(&a, &p, n);
    assert!(okh);
    let mut l = 0;
    while l < n {
        let mut r = l;
        while r < n {
            let lv = t_lca_val(&a, &p, n, l, r);
            let mi = t_argmin(&a, l, r);
            assert_eq!(lv, a[mi]);
            r += 1;
        }
        l += 1;
    }
}

#[test]
fn test_two_elements() {
    let a = vec![5i64, 3];
    let p = cartesian_tree(&a);
    assert_eq!(p[0], 1);
    assert_eq!(p[1], 2);

    let b = vec![3i64, 5];
    let q = cartesian_tree(&b);
    assert_eq!(q[0], 2);
    assert_eq!(q[1], 0);
}

#[test]
fn test_sorted_chain() {
    let a = vec![1i64, 2, 3, 4, 5];
    let p = cartesian_tree(&a);
    assert_eq!(p[0], 5);
    assert_eq!(p[1], 0);
    assert_eq!(p[2], 1);
    assert_eq!(p[3], 2);
    assert_eq!(p[4], 3);
}

#[test]
fn test_duplicate_ties() {
    let a = vec![2i64, 1, 1, 3, 1];
    let p = cartesian_tree(&a);
    let n = a.len();
    // the leftmost minimum (index 1) is the root
    assert_eq!(p[1], n);
    let okh = t_heap_ok(&a, &p, n);
    assert!(okh);
    let mut l = 0;
    while l < n {
        let mut r = l;
        while r < n {
            let lv = t_lca_val(&a, &p, n, l, r);
            let mi = t_argmin(&a, l, r);
            assert_eq!(lv, a[mi]);
            r += 1;
        }
        l += 1;
    }
}
