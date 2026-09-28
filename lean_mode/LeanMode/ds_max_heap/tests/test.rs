#[test]
fn test_push_single() {
    let h = ds_max_heap::max_heap_push(vec![], 5);
    assert_eq!(h.len(), 1);
    assert_eq!(h[0], 5);
}

#[test]
fn test_push_sift_up() {
    let h0 = ds_max_heap::max_heap_push(vec![], 3);
    let h1 = ds_max_heap::max_heap_push(h0, 5);
    let h2 = ds_max_heap::max_heap_push(h1, 4);
    // sift-up: [5, 3, 4]
    assert_eq!(h2.len(), 3);
    assert_eq!(h2[0], 5);
    assert_eq!(h2[1], 3);
    assert_eq!(h2[2], 4);
}

#[test]
fn test_pop_max_single() {
    let (h, m) = ds_max_heap::max_heap_pop_max(vec![7]);
    assert_eq!(m, 7);
    assert_eq!(h.len(), 0);
}

#[test]
fn test_pop_max_sift_down() {
    // [9,6,8,3,5,7,4] is a valid max-heap; popping 9 sifts 4 down → [8,6,7,3,5,4]
    let (h, m) = ds_max_heap::max_heap_pop_max(vec![9, 6, 8, 3, 5, 7, 4]);
    assert_eq!(m, 9);
    assert_eq!(h.len(), 6);
    assert_eq!(h[0], 8);
    assert_eq!(h[1], 6);
    assert_eq!(h[2], 7);
    assert_eq!(h[3], 3);
    assert_eq!(h[4], 5);
    assert_eq!(h[5], 4);
}

#[test]
fn test_push_many_pop_all_descending() {
    let mut h: Vec<i64> = vec![];
    h = ds_max_heap::max_heap_push(h, 3);
    h = ds_max_heap::max_heap_push(h, 1);
    h = ds_max_heap::max_heap_push(h, 4);
    h = ds_max_heap::max_heap_push(h, 1);
    h = ds_max_heap::max_heap_push(h, 5);
    h = ds_max_heap::max_heap_push(h, 9);
    h = ds_max_heap::max_heap_push(h, 2);
    h = ds_max_heap::max_heap_push(h, 6);

    // pop all → descending 9,6,5,4,3,2,1,1
    let (h1, m1) = ds_max_heap::max_heap_pop_max(h);
    assert_eq!(m1, 9);
    let (h2, m2) = ds_max_heap::max_heap_pop_max(h1);
    assert_eq!(m2, 6);
    let (h3, m3) = ds_max_heap::max_heap_pop_max(h2);
    assert_eq!(m3, 5);
    let (h4, m4) = ds_max_heap::max_heap_pop_max(h3);
    assert_eq!(m4, 4);
    let (h5, m5) = ds_max_heap::max_heap_pop_max(h4);
    assert_eq!(m5, 3);
    let (h6, m6) = ds_max_heap::max_heap_pop_max(h5);
    assert_eq!(m6, 2);
    let (h7, m7) = ds_max_heap::max_heap_pop_max(h6);
    assert_eq!(m7, 1);
    let (h8, m8) = ds_max_heap::max_heap_pop_max(h7);
    assert_eq!(m8, 1);
    assert_eq!(h8.len(), 0);
}

#[test]
fn test_push_preserves_heap_property() {
    // push 3,1,4 then check every node is ≤ its parent
    let h0 = ds_max_heap::max_heap_push(vec![], 3);
    let h1 = ds_max_heap::max_heap_push(h0, 1);
    let h2 = ds_max_heap::max_heap_push(h1, 4);
    assert!(h2[0] >= h2[1]);
    assert!(h2[0] >= h2[2]);
}
