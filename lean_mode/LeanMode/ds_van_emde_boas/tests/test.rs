use ds_van_emde_boas::{veb_delete, veb_insert, veb_max, veb_member, veb_min, veb_new, veb_successor, Veb};

/// Build a vEB tree over universe `16` from `a[start .. end)`.
fn t_build(a: &[usize], start: usize, end: usize) -> Veb {
    let mut t = veb_new(16);
    let mut i = start;
    while i < end {
        t = veb_insert(t, a[i]);
        i += 1;
    }
    t
}

/// Number of members of `t` within the half-open range `[lo, hi)`.
fn t_count(t: &Veb, lo: usize, hi: usize) -> usize {
    let mut c: usize = 0;
    let mut i = lo;
    while i < hi {
        if veb_member(t, i) {
            c += 1;
        }
        i += 1;
    }
    c
}

#[test]
fn test_insert_member() {
    let a = vec![3usize, 11, 7, 0, 15];
    let t = t_build(&a, 0, 5);
    assert_eq!(t_count(&t, 0, 16), 5);
    assert!(veb_member(&t, 3));
    assert!(veb_member(&t, 15));
    assert!(!veb_member(&t, 5));
}

#[test]
fn test_min_max_successor() {
    let a = vec![3usize, 11, 7];
    let t = t_build(&a, 0, 3);
    let (ma, mv) = veb_min(&t);
    assert!(ma);
    assert_eq!(mv, 3);
    let (xa, xv) = veb_max(&t);
    assert!(xa);
    assert_eq!(xv, 11);
    let (f1, s1) = veb_successor(&t, 3);
    assert!(f1);
    assert_eq!(s1, 7);
    let (f2, s2) = veb_successor(&t, 7);
    assert!(f2);
    assert_eq!(s2, 11);
    let (f3, s3) = veb_successor(&t, 11);
    assert!(!f3);
    assert_eq!(s3, 0);
}

#[test]
fn test_delete() {
    let a = vec![3usize, 11, 7];
    let mut t = t_build(&a, 0, 3);
    t = veb_delete(t, 3);
    assert!(!veb_member(&t, 3));
    assert_eq!(t_count(&t, 0, 16), 2);
    let (ma, mv) = veb_min(&t);
    assert!(ma);
    assert_eq!(mv, 7);
    t = veb_delete(t, 7);
    t = veb_delete(t, 11);
    assert_eq!(t_count(&t, 0, 16), 0);
    let (ea, ev) = veb_min(&t);
    assert!(!ea);
    assert_eq!(ev, 0);
}

#[test]
fn test_empty_tree() {
    let t = veb_new(16);
    assert!(!veb_member(&t, 0));
    assert!(!veb_member(&t, 9));
    let (a, am) = veb_min(&t);
    assert!(!a);
    assert_eq!(am, 0);
    let (f, s) = veb_successor(&t, 0);
    assert!(!f);
    assert_eq!(s, 0);
}
