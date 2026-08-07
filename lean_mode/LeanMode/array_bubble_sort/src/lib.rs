//! Pure bubble sort — returns a new sorted `Vec<f64>`.

/// Sorts the given slice using bubble sort, returning a new `Vec<f64>`.
pub fn bubble_sort(input: &[f64]) -> Vec<f64> {
    let mut arr = input.to_vec();
    let len = arr.len();
    if len <= 1 {
        return arr;
    }
    let mut i = 0;
    while i < len {
        let mut swapped = false;
        let mut j = 0;
        while j + 1 < len - i {
            let a = arr[j];
            let b = arr[j + 1];
            if a > b {
                arr.swap(j, j + 1);
                swapped = true;
            }
            j += 1;
        }
        if !swapped {
            break;
        }
        i += 1;
    }
    arr
}
