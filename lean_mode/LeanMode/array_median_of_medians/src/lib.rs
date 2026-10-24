//! Linear-time selection via the median-of-medians algorithm (BFPRT).
//!
//! `select_kth` returns the k-th smallest element (0-indexed) of its input,
//! using the classic "median of medians" pivot so that the worst case is O(n).

/// `n / 2` on a `usize` (routed through a helper so the arithmetic is tracked).
fn usize_half(n: usize) -> usize {
    n / 2
}

/// `a + b` on `usize` values (routed through a helper so the arithmetic is tracked).
fn usize_add(a: usize, b: usize) -> usize {
    a + b
}

/// `a - b` on `usize` values (routed through a helper so the arithmetic is tracked).
fn usize_sub(a: usize, b: usize) -> usize {
    a - b
}

/// Sort `arr` in place using insertion sort (meant for small groups, ≤ 5 elements).
fn insertion_sort(arr: Vec<i32>) -> Vec<i32> {
    let n = arr.len();
    let mut a = arr;
    let mut i = 1;
    while i < n {
        let key = a[i];
        let mut j = i;
        let mut done = false;
        while j > 0 && !done {
            let prev = a[j - 1];
            if prev > key {
                a[j] = prev;
                j -= 1;
            } else {
                done = true;
            }
        }
        a[j] = key;
        i += 1;
    }
    a
}

/// Select the k-th smallest element (0-indexed) of `arr` using the
/// median-of-medians algorithm.  Requires `k < arr.len()`.
fn select(arr: Vec<i32>, k: usize) -> i32 {
    let n = arr.len();
    if n <= 5 {
        let sorted = insertion_sort(arr);
        sorted[k]
    } else {
        // Collect the (lower) median of each consecutive group of 5.
        let mut medians: Vec<i32> = Vec::new();
        let mut i = 0;
        while i < n {
            let mut group: Vec<i32> = Vec::new();
            let mut c = 0;
            while c < 5 && i < n {
                group.push(arr[i]);
                i += 1;
                c += 1;
            }
            let sorted_group = insertion_sort(group);
            let glen = sorted_group.len();
            let mid = if glen >= 5 { 2 } else if glen >= 3 { 1 } else { 0 };
            medians.push(sorted_group[mid]);
        }

        // Pivot = median of the medians.
        let ml = usize_half(medians.len());
        let pivot = select(medians, ml);

        // Three-way partition around the pivot.
        let mut less: Vec<i32> = Vec::new();
        let mut equal: Vec<i32> = Vec::new();
        let mut greater: Vec<i32> = Vec::new();
        let mut j = 0;
        while j < n {
            let x = arr[j];
            if x < pivot {
                less.push(x);
            } else if x == pivot {
                equal.push(x);
            } else {
                greater.push(x);
            }
            j += 1;
        }

        let le = usize_add(less.len(), equal.len());
        if k < less.len() {
            select(less, k)
        } else if k < le {
            pivot
        } else {
            select(greater, usize_sub(k, le))
        }
    }
}

/// Return the k-th smallest element (0-indexed) of `input`.
///
/// Requires `k < input.len()`.  Runs in O(n) worst-case time.
pub fn select_kth(input: &[i32], k: usize) -> i32 {
    let arr = input.to_vec();
    select(arr, k)
}
