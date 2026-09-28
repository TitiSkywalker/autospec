use ds_queue::{queue_new, enqueue, dequeue, peek, Queue};

/// Helper: build a queue from a slice by enqueuing in order.
fn build_queue(values: &[i32]) -> Queue {
    let mut q = queue_new();
    let mut i: usize = 0;
    while i < values.len() {
        q = enqueue(q, values[i]);
        i += 1;
    }
    q
}

/// Helper: drain the queue front-to-back into a Vec.
fn drain(q: Queue, out: &mut Vec<i32>) {
    let (q2, v) = dequeue(q);
    match v {
        Some(x) => {
            out.push(x);
            drain(q2, out);
        }
        None => {}
    }
}

#[test]
fn test_empty_dequeue_none() {
    let q = queue_new();
    let (_, v) = dequeue(q);
    let is_none = match v { Some(_) => false, None => true };
    assert!(is_none);
}

#[test]
fn test_fifo_order() {
    let q = build_queue(&[1, 2, 3]);
    let mut out: Vec<i32> = Vec::new();
    drain(q, &mut out);
    assert_eq!(out.len(), 3);
    assert_eq!(out[0], 1);
    assert_eq!(out[1], 2);
    assert_eq!(out[2], 3);
}

#[test]
fn test_peek_front() {
    let q = build_queue(&[7, 8]);
    let front = match peek(&q) { Some(x) => x, None => -1 };
    assert_eq!(front, 7);
}

#[test]
fn test_peek_empty_none() {
    let q = queue_new();
    let v = peek(&q);
    let is_none = match v { Some(_) => false, None => true };
    assert!(is_none);
}

#[test]
fn test_grow() {
    // forces several doublings (0 -> 1 -> 2 -> 4 -> 8)
    let q = build_queue(&[10, 20, 30, 40, 50, 60]);
    let mut out: Vec<i32> = Vec::new();
    drain(q, &mut out);
    assert_eq!(out.len(), 6);
    assert_eq!(out[0], 10);
    assert_eq!(out[1], 20);
    assert_eq!(out[2], 30);
    assert_eq!(out[3], 40);
    assert_eq!(out[4], 50);
    assert_eq!(out[5], 60);
}

#[test]
fn test_wraparound() {
    // enqueue 4, dequeue 2 (head wraps), enqueue 2 more, then drain
    let q0 = build_queue(&[1, 2, 3, 4]);
    let (q1, _) = dequeue(q0);
    let (q2, _) = dequeue(q1);
    let q3 = enqueue(q2, 5);
    let q4 = enqueue(q3, 6);
    let mut out: Vec<i32> = Vec::new();
    drain(q4, &mut out);
    assert_eq!(out.len(), 4);
    assert_eq!(out[0], 3);
    assert_eq!(out[1], 4);
    assert_eq!(out[2], 5);
    assert_eq!(out[3], 6);
}
