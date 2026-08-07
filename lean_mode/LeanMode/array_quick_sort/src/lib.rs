//! Pure quicksort — returns a new sorted `Vec<i32>`.

/// Partitions the array around a pivot element for quicksort.
fn partition(arr: &mut [i32], lo: usize, hi: usize) -> usize {
    let pivot = hi;
    let pivot_val = arr[pivot];
    let mut i = lo;
    let mut j = hi - 1;

    loop {
        loop {
            if i > hi {
                break;
            }
            let val = arr[i];
            if val >= pivot_val {
                break;
            }
            i += 1;
        }
        loop {
            if j == 0 {
                break;
            }
            let val = arr[j];
            if val <= pivot_val {
                break;
            }
            j -= 1;
        }
        if j == 0 || i >= j {
            break;
        }
        let val_i = arr[i];
        let val_j = arr[j];
        if val_i == val_j {
            i += 1;
            j -= 1;
        } else {
            arr.swap(i, j);
        }
    }
    arr.swap(i, pivot);
    i
}

/// Internal quicksort implementation.
fn _quick_sort(arr: &mut [i32], mut lo: usize, mut hi: usize) {
    while lo < hi {
        let pivot = partition(arr, lo, hi);

        if pivot - lo < hi - pivot {
            if pivot > 0 {
                _quick_sort(arr, lo, pivot - 1);
            }
            lo = pivot + 1;
        } else {
            _quick_sort(arr, pivot + 1, hi);
            hi = pivot - 1;
        }
    }
}

/// Sorts the given slice using quicksort, returning a new `Vec<i32>`.
pub fn quick_sort(input: &[i32]) -> Vec<i32> {
    let mut arr = input.to_vec();
    let len = arr.len();
    if len > 1 {
        _quick_sort(&mut arr, 0, len - 1);
    }
    arr
}
