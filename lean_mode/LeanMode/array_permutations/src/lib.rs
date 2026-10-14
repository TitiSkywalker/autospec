//! Generate all permutations of `0..n-1` by recursive backtracking.

/// Mutable state threaded through the backtracking recursion by value
/// (AENEAS models `&mut` as consume-and-return, so we spell that out).
struct BacktrackState {
    /// Which values are currently placed in the prefix.
    used: Vec<bool>,
    /// The permutation under construction.
    current: Vec<usize>,
    /// Flat output: every finished permutation concatenated.
    result: Vec<usize>,
}

/// Recursive helper: fill position `k` of `current` with each not-yet-used
/// value in turn, recurse, then undo the choice (true backtracking). The base
/// case `k == n` appends the finished `current` to `result`.
fn backtrack(mut state: BacktrackState, n: usize, k: usize) -> BacktrackState {
    if k == n {
        let mut i = 0;
        while i < n {
            let x = state.current[i];
            state.result.push(x);
            i += 1;
        }
        state
    } else {
        let mut v = 0;
        while v < n {
            if !state.used[v] {
                state.used[v] = true;
                state.current[k] = v;
                state = backtrack(state, n, k + 1);
                state.used[v] = false;
            }
            v += 1;
        }
        state
    }
}

/// Return all permutations of `0..n-1` as a flat list: block `b` (of length
/// `n`) holds the `b`-th permutation, in lexicographic order.
pub fn permutations(n: usize) -> Vec<usize> {
    let used: Vec<bool> = vec![false; n];
    let current: Vec<usize> = vec![0usize; n];
    let result: Vec<usize> = Vec::new();
    let state = BacktrackState {
        used,
        current,
        result,
    };
    let final_state = backtrack(state, n, 0);
    final_state.result
}
