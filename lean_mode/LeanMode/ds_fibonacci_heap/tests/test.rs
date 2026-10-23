use ds_fibonacci_heap::{fib_extract_min, fib_find_min, fib_insert, fib_merge, fib_new, FibHeap};

/// The minimum key of `h`, or the sentinel `-1` when the heap is empty.
fn q_min(h: &FibHeap) -> i64 {
    match fib_find_min(h) {
        Some(x) => x,
        None => -1,
    }
}

/// Extract `n` keys from `h` into `out` (the sentinel `-1` is recorded if the
/// heap turns out to be smaller) and return the remaining heap.
fn q_drain(h: FibHeap, n: usize, out: &mut Vec<i64>) -> FibHeap {
    let mut h = h;
    let mut i: usize = 0;
    while i < n {
        let (h2, k) = fib_extract_min(h);
        h = h2;
        let x = match k {
            Some(y) => y,
            None => -1,
        };
        out.push(x);
        i += 1;
    }
    h
}

#[test]
fn test_insert_extract_sorted() {
    let mut h = fib_new(8);
    h = fib_insert(h, 5);
    h = fib_insert(h, 3);
    h = fib_insert(h, 9);
    h = fib_insert(h, 1);
    h = fib_insert(h, 7);
    assert_eq!(q_min(&h), 1);
    let mut out: Vec<i64> = Vec::new();
    let _h2 = q_drain(h, 5, &mut out);
    assert_eq!(out[0], 1);
    assert_eq!(out[1], 3);
    assert_eq!(out[2], 5);
    assert_eq!(out[3], 7);
    assert_eq!(out[4], 9);
}

#[test]
fn test_duplicates_and_empty_heap() {
    let e = fib_new(4);
    assert_eq!(q_min(&e), -1);
    let mut h = fib_new(8);
    h = fib_insert(h, 4);
    h = fib_insert(h, 4);
    h = fib_insert(h, 6);
    h = fib_insert(h, 2);
    assert_eq!(q_min(&h), 2);
    let mut out: Vec<i64> = Vec::new();
    let _h2 = q_drain(h, 4, &mut out);
    assert_eq!(out[0], 2);
    assert_eq!(out[1], 4);
    assert_eq!(out[2], 4);
    assert_eq!(out[3], 6);
}

#[test]
fn test_merge_two_heaps() {
    let mut a = fib_new(10);
    a = fib_insert(a, 10);
    a = fib_insert(a, 30);
    let mut b = fib_new(10);
    b = fib_insert(b, 20);
    b = fib_insert(b, 5);
    let c = fib_merge(a, b);
    assert_eq!(q_min(&c), 5);
    let mut out: Vec<i64> = Vec::new();
    let _c2 = q_drain(c, 4, &mut out);
    assert_eq!(out[0], 5);
    assert_eq!(out[1], 10);
    assert_eq!(out[2], 20);
    assert_eq!(out[3], 30);
}

#[test]
fn test_many_keys_drain_in_order() {
    let mut h = fib_new(16);
    let keys = vec![8, 3, 12, 1, 9, 5, 14, 2, 7, 11, 4, 6];
    let mut i: usize = 0;
    while i < keys.len() {
        let k = keys[i];
        h = fib_insert(h, k);
        i += 1;
    }
    assert_eq!(q_min(&h), 1);
    let mut out: Vec<i64> = Vec::new();
    let _h2 = q_drain(h, 12, &mut out);
    // the keys come out in non-decreasing order, starting at the minimum
    let mut ok = true;
    let mut j: usize = 1;
    while j < out.len() {
        let x = out[j - 1];
        let y = out[j];
        if x > y {
            ok = false;
        }
        j += 1;
    }
    assert!(ok);
    assert_eq!(out[0], 1);
    assert_eq!(out[11], 14);
}
