//! Double-ended queue (deque) as a growable ring buffer.
//!
//! A deque backed by a flat `Vec<i32>` with a circular `head`/`len` layout.
//! The buffer doubles when full, so `push_front` / `push_back` never fail.

/// A growable ring-buffer deque.
pub struct Deque {
    pub buf: Vec<i32>,
    pub head: usize,
    pub len: usize,
}

/// An empty deque.
pub fn deque_new() -> Deque {
    Deque {
        buf: Vec::new(),
        head: 0,
        len: 0,
    }
}

/// Re-layout the deque contiguously into a doubled-capacity buffer
/// (head becomes 0, elements occupy `buf[0 .. len]`).
fn grow(d: Deque) -> Deque {
    let cap = d.buf.len();
    let new_cap = if cap == 0 { 1 } else { cap * 2 };
    let mut nb: Vec<i32> = vec![0; new_cap];
    let mut k: usize = 0;
    while k < d.len {
        let t = d.head + k;
        let src = if t >= cap { t - cap } else { t };
        nb[k] = d.buf[src];
        k += 1;
    }
    Deque {
        buf: nb,
        head: 0,
        len: d.len,
    }
}

/// Add `v` to the back.
pub fn push_back(d: Deque, v: i32) -> Deque {
    let cap = d.buf.len();
    if d.len == cap {
        let mut r = grow(d);
        let w = r.len;
        r.buf[w] = v;
        let nl = r.len + 1;
        r.len = nl;
        r
    } else {
        let mut r = d;
        let t = r.head + r.len;
        let idx = if t >= cap { t - cap } else { t };
        r.buf[idx] = v;
        let nl = r.len + 1;
        r.len = nl;
        r
    }
}

/// Add `v` to the front.
pub fn push_front(d: Deque, v: i32) -> Deque {
    let cap = d.buf.len();
    if d.len == cap {
        let mut r = grow(d);
        let nc = r.buf.len();
        let idx = if r.head == 0 { nc - 1 } else { r.head - 1 };
        r.buf[idx] = v;
        r.head = idx;
        let nl = r.len + 1;
        r.len = nl;
        r
    } else {
        let mut r = d;
        let idx = if r.head == 0 { cap - 1 } else { r.head - 1 };
        r.buf[idx] = v;
        r.head = idx;
        let nl = r.len + 1;
        r.len = nl;
        r
    }
}

/// Remove and return the front element.
pub fn pop_front(d: Deque) -> (Deque, Option<i32>) {
    let mut r = d;
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

/// Remove and return the back element.
pub fn pop_back(d: Deque) -> (Deque, Option<i32>) {
    let mut r = d;
    if r.len == 0 {
        (r, None)
    } else {
        let t = r.head + r.len - 1;
        let cap = r.buf.len();
        let idx = if t >= cap { t - cap } else { t };
        let v = r.buf[idx];
        let nl = r.len - 1;
        r.len = nl;
        (r, Some(v))
    }
}
