use ds_binomial_heap::{binom_extract_min, binom_find_min, binom_insert, binom_merge, binom_new, BinomHeap};

/// Build a heap by inserting the keys `keys[start .. end]`.
fn tb_build(keys: &[i64], start: usize, end: usize) -> BinomHeap {
    let mut h = binom_new();
    let mut i = start;
    while i < end {
        let k = keys[i];
        h = binom_insert(h, k);
        i += 1;
    }
    h
}

#[test]
fn test_find_min() {
    let keys: Vec<i64> = vec![5, 3, 7, 2, 4, 9, 1];
    let h = tb_build(&keys, 0, 7);
    assert_eq!(binom_find_min(&h), 1);
}

#[test]
fn test_extract_ascending() {
    let keys: Vec<i64> = vec![5, 3, 7, 2, 4];
    let mut h = tb_build(&keys, 0, 5);
    let (k1, h1) = binom_extract_min(h);
    h = h1;
    assert_eq!(k1, 2);
    let (k2, h2) = binom_extract_min(h);
    h = h2;
    assert_eq!(k2, 3);
    let (k3, h3) = binom_extract_min(h);
    h = h3;
    assert_eq!(k3, 4);
    let (k4, h4) = binom_extract_min(h);
    h = h4;
    assert_eq!(k4, 5);
    let (k5, _h5) = binom_extract_min(h);
    assert_eq!(k5, 7);
}

#[test]
fn test_merge_min() {
    let a: Vec<i64> = vec![10, 20, 30];
    let b: Vec<i64> = vec![6, 16];
    let ha = tb_build(&a, 0, 3);
    let hb = tb_build(&b, 0, 2);
    let h = binom_merge(ha, hb);
    assert_eq!(binom_find_min(&h), 6);
    let (k, _h2) = binom_extract_min(h);
    assert_eq!(k, 6);
}

#[test]
fn test_duplicates() {
    let keys: Vec<i64> = vec![4, 4, 2, 2, 6];
    let h = tb_build(&keys, 0, 5);
    let (k1, h1) = binom_extract_min(h);
    assert_eq!(k1, 2);
    let (k2, h2) = binom_extract_min(h1);
    assert_eq!(k2, 2);
    let (k3, _h3) = binom_extract_min(h2);
    assert_eq!(k3, 4);
}
