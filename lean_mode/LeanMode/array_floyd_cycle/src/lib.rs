//! Floyd's cycle-finding algorithm (tortoise and hare) on a functional graph.
//!
//! `f[i]` is the successor of node `i`.  Starting from `x0`, the orbit of `x0`
//! eventually enters a cycle (a finite functional graph always does).  This
//! returns the *entry node* of that cycle — the first node along the orbit that
//! repeats.

/// Return the entry node of the cycle reachable from `x0` in the functional
/// graph `f`.
///
/// Precondition: `x0 < f.len()` and `f[i] < f.len()` for all `i < f.len()`,
/// so every node's successor is a valid index.
pub fn floyd_find_cycle(f: &[usize], x0: usize) -> usize {
    let mut slow = f[x0];
    let mut fast = f[f[x0]];

    // Phase 1: advance until the tortoise and hare meet somewhere on the cycle.
    while slow != fast {
        slow = f[slow];
        fast = f[f[fast]];
    }

    // Phase 2: reset one pointer to the start and step both by one; they meet
    // at the cycle entry (the distance equation mu == k*lambda - a).
    let mut mu = x0;
    while mu != slow {
        mu = f[mu];
        slow = f[slow];
    }
    mu
}
