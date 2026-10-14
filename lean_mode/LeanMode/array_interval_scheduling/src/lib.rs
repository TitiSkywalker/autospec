//! Interval scheduling — maximum number of pairwise non-overlapping intervals
//! via the greedy earliest-finish-time strategy.

/// Return the maximum number of pairwise non-overlapping intervals.
///
/// Intervals are given as parallel `starts` / `ends` arrays with
/// `starts[i] < ends[i]`. The greedy strategy sorts by finish time and always
/// takes the earliest-finishing interval that starts at or after the last one
/// selected, which is optimal for interval scheduling.
pub fn interval_scheduling(starts: &[usize], ends: &[usize]) -> usize {
    let n = starts.len();

    // Order indices by increasing finish time (bubble sort).
    let mut idx: Vec<usize> = vec![0usize; n];
    let mut k = 0;
    while k < n {
        idx[k] = k;
        k += 1;
    }
    let mut a = 0;
    while a < n {
        let mut b = 0;
        while b + 1 < n {
            let ia = idx[b];
            let ib = idx[b + 1];
            let e1 = ends[ia];
            let e2 = ends[ib];
            if e1 > e2 {
                idx[b] = ib;
                idx[b + 1] = ia;
            }
            b += 1;
        }
        a += 1;
    }

    // Greedy scan: take the earliest-finishing compatible interval each time.
    let mut count = 0;
    let mut last_end = 0;
    let mut i = 0;
    while i < n {
        let si = idx[i];
        let s = starts[si];
        let e = ends[si];
        if s >= last_end {
            count += 1;
            last_end = e;
        }
        i += 1;
    }
    count
}
