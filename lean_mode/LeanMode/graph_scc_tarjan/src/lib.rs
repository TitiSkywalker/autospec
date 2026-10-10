//! Tarjan's strongly-connected-components algorithm (recursive DFS).
//!
//! The graph is a CSR-style directed edge list `(n, from_list, to_list)`,
//! matching the shape used by `graph_topological_sort` / `graph_bfs`.
//! This is the recursive-DFS graph paradigm: `strongconnect` recurses over
//! neighbours and threads its entire mutable state by value (AENEAS models
//! `&mut` as consume-and-return, so we spell that out explicitly).

/// Mutable state threaded through Tarjan's recursion.
struct TarjanState {
    /// Discovery order; `index[v] == n` means "not yet visited".
    index: Vec<usize>,
    /// Lowest discovery index reachable from `v` (including via back edges).
    lowlink: Vec<usize>,
    /// Whether `v` currently sits on the DFS stack.
    on_stack: Vec<bool>,
    /// The DFS stack (a pre-allocated buffer plus a cursor).
    stack: Vec<usize>,
    stack_size: usize,
    /// Output: `comp[v]` = component id of `v` (dense, from 0 upward).
    comp: Vec<usize>,
    /// Next discovery index to assign.
    counter: usize,
    /// Next component id to assign.
    comp_count: usize,
}

/// Recursive Tarjan step on vertex `v`, returning the updated state.
fn strongconnect(
    n: usize,
    from_list: &[usize],
    to_list: &[usize],
    mut state: TarjanState,
    v: usize,
) -> TarjanState {
    let m = from_list.len();

    // Assign discovery index and lowlink.
    let idx_v = state.counter;
    state.index[v] = idx_v;
    state.lowlink[v] = idx_v;
    let c = idx_v + 1;
    state.counter = c;

    // Push `v` onto the DFS stack.
    let sz = state.stack_size;
    state.stack[sz] = v;
    let sz1 = sz + 1;
    state.stack_size = sz1;
    state.on_stack[v] = true;

    // Explore every neighbour `w` of `v`.
    let mut k: usize = 0;
    while k < m {
        if from_list[k] == v {
            let w = to_list[k];
            if state.index[w] == n {
                // Tree edge: recurse, then fold `w`'s lowlink into `v`'s.
                state = strongconnect(n, from_list, to_list, state, w);
                let a = state.lowlink[v];
                let b = state.lowlink[w];
                let m1 = if a < b { a } else { b };
                state.lowlink[v] = m1;
            } else {
                // Back/cross edge to a vertex still on the stack.
                if state.on_stack[w] {
                    let a = state.lowlink[v];
                    let b = state.index[w];
                    let m2 = if a < b { a } else { b };
                    state.lowlink[v] = m2;
                }
            }
        }
        k += 1;
    }

    // Root of an SCC: pop every vertex down to (and including) `v`.
    if state.lowlink[v] == state.index[v] {
        let mut done: bool = false;
        while !done {
            let sz = state.stack_size;
            let top_idx = sz - 1;
            let top = state.stack[top_idx];
            state.stack_size = top_idx;
            state.on_stack[top] = false;
            state.comp[top] = state.comp_count;
            if top == v {
                done = true;
            }
        }
        let cc = state.comp_count + 1;
        state.comp_count = cc;
    }

    state
}

/// Compute the strongly-connected-components decomposition of a directed graph.
///
/// Returns a flat `comp` vector where `comp[v]` is the component id of `v`
/// (dense ids `0, 1, …`); `comp[u] == comp[v]` iff `u` and `v` are mutually
/// reachable (in the same SCC).
pub fn scc_tarjan(n: usize, from_list: &[usize], to_list: &[usize]) -> Vec<usize> {
    let mut on_stack: Vec<bool> = Vec::new();
    let mut i: usize = 0;
    while i < n {
        on_stack.push(false);
        i += 1;
    }

    let mut state = TarjanState {
        index: vec![n; n],
        lowlink: vec![n; n],
        on_stack,
        stack: vec![0usize; n],
        stack_size: 0,
        comp: vec![n; n],
        counter: 0,
        comp_count: 0,
    };

    let mut v: usize = 0;
    while v < n {
        if state.index[v] == n {
            state = strongconnect(n, from_list, to_list, state, v);
        }
        v += 1;
    }

    state.comp
}
