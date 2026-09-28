//! Binary max-heap (array-based, O(log n) push / pop-max).
//!
//! The heap is a flat `Vec<i64>` in the standard complete-binary-tree layout:
//! `parent(i) = (i - 1) / 2`, left child `2i + 1`, right child `2i + 2`.
//! Invariant: every element is ≤ its parent.

/// Insert `v` into `heap`, maintaining the max-heap invariant.
pub fn max_heap_push(heap: Vec<i64>, v: i64) -> Vec<i64> {
    let mut h = heap;
    h.push(v);

    let mut i = h.len() - 1;
    while i > 0 {
        let p = (i - 1) / 2;
        let hi = h[i];
        let hp = h[p];
        if hi > hp {
            h.swap(i, p);
            i = p;
        } else {
            i = 0;
        }
    }
    h
}

/// Remove and return the maximum element, maintaining the max-heap invariant.
///
/// Requires `heap` to be non-empty.
pub fn max_heap_pop_max(heap: Vec<i64>) -> (Vec<i64>, i64) {
    let n = heap.len();
    let max_val = heap[0];
    let last = heap[n - 1];

    // New heap of size n-1: the old last element becomes the root, followed by
    // the old heap[1 .. n-2].
    let mut h: Vec<i64> = Vec::new();
    if n > 1 {
        h.push(last);
    }
    let mut k: usize = 1;
    while k < n - 1 {
        h.push(heap[k]);
        k += 1;
    }

    // Sift the root down.
    let size = h.len();
    let mut idx: usize = 0;
    let mut done: bool = false;
    while !done {
        let l = 2 * idx + 1;
        let r = 2 * idx + 2;
        let mut largest = idx;
        if l < size {
            if h[l] > h[largest] {
                largest = l;
            }
        }
        if r < size {
            if h[r] > h[largest] {
                largest = r;
            }
        }
        if largest == idx {
            done = true;
        } else {
            h.swap(idx, largest);
            idx = largest;
        }
    }

    (h, max_val)
}
