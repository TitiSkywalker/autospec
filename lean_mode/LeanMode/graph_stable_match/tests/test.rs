#[test]
fn test_single() {
    let rank_m = vec![0];
    let rank_w = vec![0];
    let matched = graph_stable_match::stable_match(1, &rank_m, &rank_w);
    assert_eq!(matched[0], 1);
}

#[test]
fn test_mutual_top() {
    // m0 prefers w0; m1 prefers w1; w0 prefers m0; w1 prefers m1.
    // rank_m[0]=[0,1] (m0: w0 then w1), rank_m[1]=[1,0] (m1: w1 then w0)
    let rank_m = vec![0, 1, 1, 0];
    let rank_w = vec![0, 1, 1, 0];
    let matched = graph_stable_match::stable_match(2, &rank_m, &rank_w);
    assert_eq!(matched[0], 1); // m0-w0
    assert_eq!(matched[3], 1); // m1-w1
}

#[test]
fn test_rejection_reroute() {
    // Both men prefer w0; w0 prefers m1 over m0; w1 prefers m0 over m1.
    // rank_m: m0=[0,1], m1=[0,1].  rank_w: w0=[1,0] (prefers m1), w1=[0,1] (prefers m0).
    let rank_m = vec![0, 1, 0, 1];
    let rank_w = vec![1, 0, 0, 1];
    let matched = graph_stable_match::stable_match(2, &rank_m, &rank_w);
    // m0 proposes w0 (accepted), m1 proposes w0 (w0 switches to m1),
    // m0 re-proposes w1 (accepted).  Result: m1-w0, m0-w1.
    assert_eq!(matched[2], 1); // m1-w0
    assert_eq!(matched[1], 1); // m0-w1
}

#[test]
fn test_3x3() {
    // A 3x3 case; just verify the output is a perfect matching (3 edges).
    let rank_m = vec![0, 1, 2, 1, 2, 0, 2, 0, 1];
    let rank_w = vec![2, 0, 1, 0, 1, 2, 1, 2, 0];
    let matched = graph_stable_match::stable_match(3, &rank_m, &rank_w);
    let mut count = 0;
    let mut i = 0;
    while i < 9 {
        count = count + matched[i];
        i += 1;
    }
    assert_eq!(count, 3);
}
