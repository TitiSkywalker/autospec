//! FIFO queue as a growable ring buffer.
//!
//! A queue backed by a flat `Vec<i32>` with a circular `head`/`len` layout.
//! The buffer grows (doubling) when full, so `enqueue` never fails.

/// A growable ring-buffer queue.
pub struct Queue {
    pub buf: Vec<i32>,
    pub head: usize,
    pub len: usize,
}

/// An empty queue.
pub fn queue_new() -> Queue {
    Queue {
        buf: Vec::new(),
        head: 0,
        len: 0,
    }
}

/// Append `v` to the back of `q`.
pub fn enqueue(q: Queue, v: i32) -> Queue {
    let mut r = q;
    let cap = r.buf.len();
    if r.len == cap {
        // grow: double capacity (at least 1), copy the circular layout to
        // a contiguous [head=0 .. len] prefix, then append.
        let new_cap = if cap == 0 { 1 } else { cap * 2 };
        let mut nb: Vec<i32> = vec![0; new_cap];
        let mut k: usize = 0;
        while k < r.len {
            let src = (r.head + k) % cap;
            nb[k] = r.buf[src];
            k += 1;
        }
        let w = r.len;
        nb[w] = v;
        r.buf = nb;
        r.head = 0;
        let nl = r.len + 1;
        r.len = nl;
        r
    } else {
        let t = r.head + r.len;
        let idx = if t >= cap { t - cap } else { t };
        r.buf[idx] = v;
        let nl = r.len + 1;
        r.len = nl;
        r
    }
}

/// Remove and return the front element (FIFO order).
pub fn dequeue(q: Queue) -> (Queue, Option<i32>) {
    let mut r = q;
    if r.len == 0 {
        (r, None)
    } else {
        let v = r.buf[r.head];
        let cap = r.buf.len();
        let t = r.head + 1;
        let nh = if t >= cap { t - cap } else { t };
        r.head = nh;
        let nl = r.len - 1;
        r.len = nl;
        (r, Some(v))
    }
}

/// Return the front element without removing it.
pub fn peek(q: &Queue) -> Option<i32> {
    if q.len == 0 {
        None
    } else {
        Some(q.buf[q.head])
    }
}
