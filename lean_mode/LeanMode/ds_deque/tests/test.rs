use ds_deque::{deque_new, push_back, push_front, pop_front, pop_back, Deque};

/// Helper: drain the deque front-to-back into a Vec.
fn drain_front(d: Deque, out: &mut Vec<i32>) {
    let (d2, v) = pop_front(d);
    match v {
        Some(x) => {
            out.push(x);
            drain_front(d2, out);
        }
        None => {}
    }
}

#[test]
fn test_push_back_fifo() {
    let d0 = deque_new();
    let d1 = push_back(d0, 1);
    let d2 = push_back(d1, 2);
    let d3 = push_back(d2, 3);
    let mut out: Vec<i32> = Vec::new();
    drain_front(d3, &mut out);
    assert_eq!(out.len(), 3);
    assert_eq!(out[0], 1);
    assert_eq!(out[1], 2);
    assert_eq!(out[2], 3);
}

#[test]
fn test_push_front_lifo() {
    let d0 = deque_new();
    let d1 = push_front(d0, 1);
    let d2 = push_front(d1, 2);
    let d3 = push_front(d2, 3);
    let mut out: Vec<i32> = Vec::new();
    drain_front(d3, &mut out);
    assert_eq!(out.len(), 3);
    assert_eq!(out[0], 3);
    assert_eq!(out[1], 2);
    assert_eq!(out[2], 1);
}

#[test]
fn test_pop_back() {
    let d0 = deque_new();
    let d1 = push_back(d0, 1);
    let d2 = push_back(d1, 2);
    let d3 = push_back(d2, 3);
    let (d4, b1) = pop_back(d3);
    let b1 = match b1 { Some(x) => x, None => -1 };
    assert_eq!(b1, 3);
    let (d5, b2) = pop_back(d4);
    let b2 = match b2 { Some(x) => x, None => -1 };
    assert_eq!(b2, 2);
    let (d6, b3) = pop_back(d5);
    let b3 = match b3 { Some(x) => x, None => -1 };
    assert_eq!(b3, 1);
    let (_, b4) = pop_back(d6);
    let none_flag = match b4 { Some(_) => false, None => true };
    assert!(none_flag);
}

#[test]
fn test_mixed_ends() {
    let d0 = deque_new();
    let d1 = push_front(d0, 1);
    let d2 = push_back(d1, 2);
    let d3 = push_front(d2, 0);
    // deque is [0, 1, 2]
    let (d4, f) = pop_front(d3);
    let f = match f { Some(x) => x, None => -1 };
    assert_eq!(f, 0);
    let (d5, b) = pop_back(d4);
    let b = match b { Some(x) => x, None => -1 };
    assert_eq!(b, 2);
    let (d6, m) = pop_front(d5);
    let m = match m { Some(x) => x, None => -1 };
    assert_eq!(m, 1);
    let (_, last) = pop_front(d6);
    let none_flag = match last { Some(_) => false, None => true };
    assert!(none_flag);
}

#[test]
fn test_grow() {
    // forces several doublings and wraps
    let mut d = deque_new();
    let mut i: usize = 0;
    while i < 10 {
        d = push_back(d, i as i32);
        i += 1;
    }
    d = push_front(d, 100);
    let mut out: Vec<i32> = Vec::new();
    drain_front(d, &mut out);
    assert_eq!(out.len(), 11);
    assert_eq!(out[0], 100);
    assert_eq!(out[1], 0);
    assert_eq!(out[10], 9);
}
