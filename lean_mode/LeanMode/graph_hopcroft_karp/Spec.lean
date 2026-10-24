import LeanMode.graph_hopcroft_karp.GraphHopcroftKarp
import Mathlib

open Aeneas Aeneas.Std Result
open graph_hopcroft_karp

set_option linter.unusedVariables false

/-!
# Specification: Maximum Bipartite Matching (Hopcroft–Karp)

`hopcroft_karp nl nr adj` returns `ok (size, matched)` where `adj` is a flat,
row-major `nl × nr` 0/1 adjacency matrix (`adj u v = 1` means left vertex `u`
is adjacent to right vertex `v`) and `matched` is a flat `nl × nr` 0/1 matrix
marking the matched edges.  `size` is the matching cardinality.

A *matching* is a 0/1 matrix whose ones lie on real edges and with at most one
`1` per row (left vertex) and per column (right vertex).  The theorems assert
that the returned matrix is a matching, that `size` equals its cardinality, and
that it is a *maximum* matching (Berge's augmenting-path theorem is the subtle
part of the proof).
-/

/-- View a flat `rows × cols` matrix (row-major: element `(row, col)` lives at
    `row * cols + col`) as `Fin rows → Fin cols → Int`. -/
def toMatrix (s : Slice Int) (rows cols : Nat) (h : s.length = rows * cols)
    : Fin rows → Fin cols → Int :=
  λ i j =>
    have h_bound : i.val * cols + j.val < s.length := by
      rw [h]
      have hi : i.val < rows := i.is_lt
      have hj : j.val < cols := j.is_lt
      calc
        i.val * cols + j.val < i.val * cols + cols := Nat.add_lt_add_left hj (i.val * cols)
        _ = (i.val + 1) * cols := by ring
        _ ≤ rows * cols := Nat.mul_le_mul_right cols (Nat.succ_le_of_lt hi)
    s[i.val * cols + j.val]'h_bound

/-- `matched` is a valid matching of the bipartite graph `adj`: 0/1 entries,
    ones only on real edges, and at most one match per left and per right
    vertex. -/
def is_matching {nl nr : Nat} (adj matched : Fin nl → Fin nr → Int) : Prop :=
  (∀ (u : Fin nl) (v : Fin nr), matched u v = 1 → adj u v = 1) ∧
  (∀ (u : Fin nl) (v : Fin nr), matched u v = 0 ∨ matched u v = 1) ∧
  (∀ (u : Fin nl) (v1 v2 : Fin nr), matched u v1 = 1 → matched u v2 = 1 → v1 = v2) ∧
  (∀ (u1 u2 : Fin nl) (v : Fin nr), matched u1 v = 1 → matched u2 v = 1 → u1 = u2)

/-- The cardinality of a matching: the number of matched edges. -/
def matching_size {nl nr : Nat} (matched : Fin nl → Fin nr → Int) : Int :=
  ∑ u : Fin nl, ∑ v : Fin nr, matched u v

/-- `matched` is a maximum matching: it is a matching, and no matching is
    larger. -/
def is_max_matching {nl nr : Nat} (adj matched : Fin nl → Fin nr → Int) : Prop :=
  is_matching adj matched ∧
  ∀ other : Fin nl → Fin nr → Int,
    is_matching adj other → matching_size other ≤ matching_size matched


/-! ## Specification theorems -/

/-- `hopcroft_karp` returns a matching matrix of length `nl * nr`. -/
theorem hopcroft_karp_length
    (nl nr : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h : hopcroft_karp nl nr adj = ok (size, matched)) :
    (alloc.vec.Vec.deref matched).length = nl.val * nr.val := by
  sorry

/-- Soundness: the returned matrix is a valid matching. -/
theorem hopcroft_karp_is_matching
    (nl nr : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = nl.val * nr.val)
    (h_adj01 : ∀ (u : Fin nl.val) (v : Fin nr.val),
       toMatrix adj nl.val nr.val h_adj u v = 0 ∨
       toMatrix adj nl.val nr.val h_adj u v = 1)
    (h_matched : (alloc.vec.Vec.deref matched).length = nl.val * nr.val)
    (h : hopcroft_karp nl nr adj = ok (size, matched)) :
    is_matching (toMatrix adj nl.val nr.val h_adj)
                (toMatrix (alloc.vec.Vec.deref matched) nl.val nr.val h_matched) := by
  sorry

/-- The returned `size` equals the cardinality of the returned matching. -/
theorem hopcroft_karp_size
    (nl nr : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = nl.val * nr.val)
    (h_matched : (alloc.vec.Vec.deref matched).length = nl.val * nr.val)
    (h : hopcroft_karp nl nr adj = ok (size, matched)) :
    size = matching_size (toMatrix (alloc.vec.Vec.deref matched) nl.val nr.val h_matched) := by
  sorry

/-- Optimality: the returned matching is maximum. -/
theorem hopcroft_karp_maximal
    (nl nr : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = nl.val * nr.val)
    (h_adj01 : ∀ (u : Fin nl.val) (v : Fin nr.val),
       toMatrix adj nl.val nr.val h_adj u v = 0 ∨
       toMatrix adj nl.val nr.val h_adj u v = 1)
    (h_matched : (alloc.vec.Vec.deref matched).length = nl.val * nr.val)
    (h : hopcroft_karp nl nr adj = ok (size, matched)) :
    is_max_matching (toMatrix adj nl.val nr.val h_adj)
                    (toMatrix (alloc.vec.Vec.deref matched) nl.val nr.val h_matched) := by
  sorry
