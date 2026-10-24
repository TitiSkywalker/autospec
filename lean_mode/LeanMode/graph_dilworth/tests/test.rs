//! Tests for the minimum vertex-disjoint path cover of a DAG.
//!
//! The brute-force cross-check `brute_path_cover` enumerates every assignment
//! of a successor (or "none") to every vertex, keeps the assignments that are
//! genuine edge sets, injective, and cycle-free, and maximises the number of
//! assigned successors.  A cycle-free injective successor assignment is
//! exactly a vertex-disjoint path cover, and its number of paths is
//! `n - assigned`, so `n - best` is the minimum path cover.

/// Brute-force minimum path cover (successor-assignment search).
fn brute_path_cover(n: usize, frm: &[usize], to: &[usize]) -> usize {
    let m = frm.len();
    // succ[u] = successor of `u` on its path, or `n` for "none".
    let mut succ: Vec<usize> = vec![0usize; n];
    let mut used: Vec<usize> = vec![0usize; n];
    let mut best: usize = 0;
    let mut done: bool = false;
    while !done {
        // Check the current successor assignment.
        let mut ok: bool = true;
        let mut i: usize = 0;
        while i < n {
            used[i] = 0;
            i += 1;
        }
        let mut count: usize = 0;
        i = 0;
        while i < n {
            let w = succ[i];
            if w != n {
                // (i, w) must be a genuine edge ...
                let mut has: bool = false;
                let mut e: usize = 0;
                while e < m {
                    if frm[e] == i && to[e] == w {
                        has = true;
                    }
                    e += 1;
                }
                if !has {
                    ok = false;
                }
                // ... and the successor map must be injective.
                if used[w] == 1 {
                    ok = false;
                }
                used[w] = 1;
                count += 1;
            }
            i += 1;
        }
        // ... and cycle-free: walking from any vertex must reach "none"
        // within `n` steps.
        i = 0;
        while i < n {
            if succ[i] != n {
                let mut cur = succ[i];
                let mut steps: usize = 0;
                while steps < n && cur != n {
                    steps += 1;
                    cur = succ[cur];
                }
                if steps == n {
                    ok = false;
                }
            }
            i += 1;
        }
        if ok {
            if count > best {
                best = count;
            }
        }
        // Odometer over `succ[0..n]` with digits `0..=n`.
        let mut j: usize = 0;
        let mut carry: bool = true;
        while j < n && carry {
            let d = succ[j] + 1;
            if d > n {
                succ[j] = 0;
            } else {
                succ[j] = d;
                carry = false;
            }
            j += 1;
        }
        if carry {
            done = true;
        }
    }
    usize_diff(n, best)
}

/// `a - b` on two `usize` parameters.
///
/// (Named `usize_diff` rather than something starting with `cover`, so that it
/// is not a prefix of any `src/lib.rs` function name: the test extractor keeps
/// definitions by name prefix and a clash would re-emit the library's own
/// helper into the test module.)
fn usize_diff(a: usize, b: usize) -> usize {
    let r = a - b;
    r
}

#[test]
fn test_empty() {
    let from: Vec<usize> = vec![];
    let to: Vec<usize> = vec![];
    assert_eq!(graph_dilworth::min_path_cover(0, &from, &to), 0);
}

#[test]
fn test_single_vertex() {
    // A lone vertex needs one (trivial) path.
    let from: Vec<usize> = vec![];
    let to: Vec<usize> = vec![];
    assert_eq!(graph_dilworth::min_path_cover(1, &from, &to), 1);
}

#[test]
fn test_three_isolated() {
    let from: Vec<usize> = vec![];
    let to: Vec<usize> = vec![];
    assert_eq!(graph_dilworth::min_path_cover(3, &from, &to), 3);
}

#[test]
fn test_chain() {
    // 0 -> 1 -> 2 -> 3: one path covers everything.
    let from = vec![0, 1, 2];
    let to = vec![1, 2, 3];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 1);
    assert_eq!(brute_path_cover(4, &from, &to), 1);
}

#[test]
fn test_transitive_tournament() {
    // i -> j for all i < j: the increasing order is a single Hamiltonian path.
    let from = vec![0, 0, 0, 1, 1, 2];
    let to = vec![1, 2, 3, 2, 3, 3];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 1);
    assert_eq!(brute_path_cover(4, &from, &to), 1);
}

#[test]
fn test_diamond() {
    // 0 -> 1 -> 3, 0 -> 2 -> 3: two paths (0,1,3) and (2).
    let from = vec![0, 0, 1, 2];
    let to = vec![1, 2, 3, 3];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 2);
    assert_eq!(brute_path_cover(4, &from, &to), 2);
}

#[test]
fn test_zigzag_needs_augmenting_path() {
    // 0 -> 2, 0 -> 3, 1 -> 2.
    // Greedily starting 0 -> 2 strands vertex 1, giving 3 paths
    // (0,2), (1), (3); the augmenting path 1 -> 2 -> 0 -> 3 re-routes to the
    // two paths (1,2) and (0,3).
    let from = vec![0, 0, 1];
    let to = vec![2, 3, 2];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 2);
    assert_eq!(brute_path_cover(4, &from, &to), 2);
}

#[test]
fn test_bipartite_like() {
    // Left {0,1,2}, right {3,4}: 0 -> 3, 0 -> 4, 1 -> 3, 1 -> 4, 2 -> 4.
    // Maximum matching 2, hence 5 - 2 = 3 paths.
    let from = vec![0, 0, 1, 1, 2];
    let to = vec![3, 4, 3, 4, 4];
    assert_eq!(graph_dilworth::min_path_cover(5, &from, &to), 3);
    assert_eq!(brute_path_cover(5, &from, &to), 3);
}

#[test]
fn test_star_out() {
    // 0 -> 1, 0 -> 2, 0 -> 3: one vertex feeds three leaves.
    let from = vec![0, 0, 0];
    let to = vec![1, 2, 3];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 3);
    assert_eq!(brute_path_cover(4, &from, &to), 3);
}

#[test]
fn test_two_chains() {
    // 0 -> 1 and 2 -> 3: two disjoint paths.
    let from = vec![0, 2];
    let to = vec![1, 3];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 2);
    assert_eq!(brute_path_cover(4, &from, &to), 2);
}

#[test]
fn test_long_augmenting_chain() {
    // 0 -> 2, 0 -> 3, 1 -> 2, 2 -> 4: the pair 1 -> 2 -> 4 forces the
    // augmenting re-route 0 -> 3, so two paths cover all five vertices.
    let from = vec![0, 0, 1, 2];
    let to = vec![2, 3, 2, 4];
    assert_eq!(graph_dilworth::min_path_cover(5, &from, &to), 2);
    assert_eq!(brute_path_cover(5, &from, &to), 2);
}

#[test]
fn test_v_shape_merge() {
    // 0 -> 2, 1 -> 2, 2 -> 3: two sources merging into one sink.
    let from = vec![0, 1, 2];
    let to = vec![2, 2, 3];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 2);
    assert_eq!(brute_path_cover(4, &from, &to), 2);
}

#[test]
fn test_isolated_vertex_with_chain() {
    // 0 -> 1 -> 2 plus the isolated vertex 3.
    let from = vec![0, 1];
    let to = vec![1, 2];
    assert_eq!(graph_dilworth::min_path_cover(4, &from, &to), 2);
    assert_eq!(brute_path_cover(4, &from, &to), 2);
}

#[test]
fn test_two_predecessors() {
    // 0 -> 1, 2 -> 1, 3 -> 4: vertex 1 has two possible predecessors.
    let from = vec![0, 2, 3];
    let to = vec![1, 1, 4];
    assert_eq!(graph_dilworth::min_path_cover(5, &from, &to), 3);
    assert_eq!(brute_path_cover(5, &from, &to), 3);
}

#[test]
fn test_augmenting_path_ends_at_start_right_copy() {
    // 1 -> 0, 1 -> 2, 2 -> 0 (a DAG).  The single path 1 -> 2 -> 0 covers all
    // three vertices, so the minimum path cover is 1 (maximum matching 2,
    // {(1,2), (2,0)}).  The augmenting path for the start vertex 2 is
    // `left 2 -> right 0 -> left 1 -> right 2`, and its endpoint `right 2` is
    // the *right copy of the start vertex itself*: a BFS that pre-marks
    // `seen[src]` misses it, under-matches, and returns 2 instead of 1.
    let from = vec![1, 1, 2];
    let to = vec![0, 2, 0];
    assert_eq!(graph_dilworth::min_path_cover(3, &from, &to), 1);
    assert_eq!(brute_path_cover(3, &from, &to), 1);
}

#[test]
fn test_underlabelled_five_vertex_dag() {
    // 0 -> 2, 0 -> 3, 1 -> 2, 1 -> 4, 3 -> 4 (a DAG).  The two paths
    // 0 -> 3 -> 4 and 1 -> 2 cover all five vertices, so the minimum is 2
    // (maximum matching 3); a label-dependent Kuhn bug returns 3 here.
    let from = vec![0, 0, 1, 1, 3];
    let to = vec![2, 3, 2, 4, 4];
    assert_eq!(graph_dilworth::min_path_cover(5, &from, &to), 2);
    assert_eq!(brute_path_cover(5, &from, &to), 2);
}

#[test]
fn test_paths_are_not_chains() {
    // 0 -> 1, 0 -> 2, 3 -> 0, 4 -> 0.
    // In the reachability order the two chains {3, 0, 1} and {4, 2} cover all
    // five vertices, but 4 -> 2 is not an edge, so paths (not chains) need
    // three: e.g. (3,0,1), (4), (2).  Maximum matching 2.
    let from = vec![0, 0, 3, 4];
    let to = vec![1, 2, 0, 0];
    assert_eq!(graph_dilworth::min_path_cover(5, &from, &to), 3);
    assert_eq!(brute_path_cover(5, &from, &to), 3);
}
