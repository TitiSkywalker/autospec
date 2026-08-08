//! Pure selection sort — returns a new sorted `Vec<i32>`.

/// Sorts the given slice using selection sort, returning a new `Vec<i32>`.
pub fn selection_sort(input: &[i32]) -> Vec<i32> {
    let mut arr = input.to_vec();
    let len = arr.len();
    if len <= 1 {
        return arr;
    }
    let mut i: usize = 0;
    while i < len - 1 {
        let mut min_idx = i;
        let mut j: usize = i + 1;
        while j < len {
            if arr[j] < arr[min_idx] {
                min_idx = j;
            }
            j += 1;
        }
        if min_idx != i {
            let tmp = arr[i];
            arr[i] = arr[min_idx];
            arr[min_idx] = tmp;
        }
        i += 1;
    }
    arr
}
