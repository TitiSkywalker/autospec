//! Pure merge sort — returns a new sorted `Vec<i32>`.

/// Merges two sorted subarrays arr[lo..mid] and arr[mid..hi] into arr[lo..hi].
fn merge(arr: &mut [i32], lo: usize, mid: usize, hi: usize) {
    let n1 = mid - lo;
    let n2 = hi - mid;
    let mut left: Vec<i32> = Vec::with_capacity(n1);
    let mut right: Vec<i32> = Vec::with_capacity(n2);
    let mut i = lo;
    while i < mid {
        left.push(arr[i]);
        i += 1;
    }
    let mut i = mid;
    while i < hi {
        right.push(arr[i]);
        i += 1;
    }
    let mut i = 0;
    let mut j = 0;
    let mut k = lo;
    while i < n1 && j < n2 {
        let lv = left[i];
        let rv = right[j];
        if lv <= rv {
            arr[k] = lv;
            i += 1;
        } else {
            arr[k] = rv;
            j += 1;
        }
        k += 1;
    }
    while i < n1 {
        arr[k] = left[i];
        i += 1;
        k += 1;
    }
    while j < n2 {
        arr[k] = right[j];
        j += 1;
        k += 1;
    }
}

/// Internal merge sort implementation.
fn _merge_sort(arr: &mut [i32], lo: usize, hi: usize) {
    let len = hi - lo;
    if len <= 1 {
        return;
    }
    let mid = lo + len / 2;
    _merge_sort(arr, lo, mid);
    _merge_sort(arr, mid, hi);
    merge(arr, lo, mid, hi);
}

/// Sorts the given slice using merge sort, returning a new `Vec<i32>`.
pub fn merge_sort(input: &[i32]) -> Vec<i32> {
    let mut arr = input.to_vec();
    let len = arr.len();
    if len > 1 {
        _merge_sort(&mut arr, 0, len);
    }
    arr
}
