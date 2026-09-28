use ds_linked_list::{push_front, pop_front, append, reverse, length, LinkedList};

/// Helper: push a slice of values so that the list reads front-to-back as the slice.
fn build_list(values: &[i32]) -> LinkedList {
    let mut l = LinkedList::Nil;
    let mut i: usize = values.len();
    while i > 0 {
        i -= 1;
        l = push_front(l, values[i]);
    }
    l
}

/// Helper: collect the list front-to-back into a Vec.
fn collect(list: LinkedList, acc: &mut Vec<i32>) {
    match list {
        LinkedList::Nil => {}
        LinkedList::Cons(head, tail) => {
            acc.push(head);
            collect(*tail, acc);
        }
    }
}

#[test]
fn test_push_pop_roundtrip() {
    let l = build_list(&[1, 2, 3]);
    let (l, v1) = pop_front(l);
    let h1 = match v1 { Some(x) => x, None => -1 };
    assert_eq!(h1, 1);
    let (l, v2) = pop_front(l);
    let h2 = match v2 { Some(x) => x, None => -1 };
    assert_eq!(h2, 2);
    let (l, v3) = pop_front(l);
    let h3 = match v3 { Some(x) => x, None => -1 };
    assert_eq!(h3, 3);
    let (l, v4) = pop_front(l);
    let h4 = match v4 { Some(x) => x, None => -1 };
    assert_eq!(h4, -1);
    assert_eq!(length(l), 0);
}

#[test]
fn test_length_empty() {
    assert_eq!(length(LinkedList::Nil), 0);
}

#[test]
fn test_length_three() {
    let l = build_list(&[1, 2, 3]);
    assert_eq!(length(l), 3);
}

#[test]
fn test_append() {
    let a = build_list(&[1, 2]);
    let b = build_list(&[3, 4]);
    let c = append(a, b);
    let mut out: Vec<i32> = Vec::new();
    collect(c, &mut out);
    assert_eq!(out.len(), 4);
    assert_eq!(out[0], 1);
    assert_eq!(out[1], 2);
    assert_eq!(out[2], 3);
    assert_eq!(out[3], 4);
}

#[test]
fn test_reverse() {
    let l = build_list(&[1, 2, 3]);
    let r = reverse(l);
    let mut out: Vec<i32> = Vec::new();
    collect(r, &mut out);
    assert_eq!(out.len(), 3);
    assert_eq!(out[0], 3);
    assert_eq!(out[1], 2);
    assert_eq!(out[2], 1);
}

#[test]
fn test_reverse_involutive() {
    let l = build_list(&[5, 6, 7, 8]);
    let r = reverse(l);
    let r2 = reverse(r);
    let mut out: Vec<i32> = Vec::new();
    collect(r2, &mut out);
    assert_eq!(out.len(), 4);
    assert_eq!(out[0], 5);
    assert_eq!(out[1], 6);
    assert_eq!(out[2], 7);
    assert_eq!(out[3], 8);
}

#[test]
fn test_append_length() {
    let a = build_list(&[1, 2]);
    let b = build_list(&[3, 4, 5]);
    let c = append(a, b);
    assert_eq!(length(c), 5);
}
