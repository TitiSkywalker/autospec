#[test]
fn test_empty() {
    let starts: Vec<usize> = vec![];
    let ends: Vec<usize> = vec![];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 0);
}

#[test]
fn test_single() {
    let starts = vec![1];
    let ends = vec![3];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 1);
}

#[test]
fn test_classic() {
    // (1,4), (3,5), (0,6), (5,7), (3,9), (5,9), (6,10), (8,11), (8,12), (2,14), (12,16)
    let starts = vec![1, 3, 0, 5, 3, 5, 6, 8, 8, 2, 12];
    let ends = vec![4, 5, 6, 7, 9, 9, 10, 11, 12, 14, 16];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 4);
}

#[test]
fn test_all_overlap() {
    let starts = vec![0, 1, 2];
    let ends = vec![10, 10, 10];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 1);
}

#[test]
fn test_all_disjoint() {
    let starts = vec![0, 2, 4, 6];
    let ends = vec![1, 3, 5, 7];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 4);
}

#[test]
fn test_nested() {
    // A long interval (0,10) and two short ones inside it (1,2), (3,4).
    let starts = vec![0, 1, 3];
    let ends = vec![10, 2, 4];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 2);
}

#[test]
fn test_same_end() {
    let starts = vec![0, 1];
    let ends = vec![5, 5];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 1);
}

#[test]
fn test_touch_at_endpoint() {
    // (0,1) and (1,2) touch at 1 but do not overlap.
    let starts = vec![0, 1];
    let ends = vec![1, 2];
    let result = array_interval_scheduling::interval_scheduling(&starts, &ends);
    assert_eq!(result, 2);
}
