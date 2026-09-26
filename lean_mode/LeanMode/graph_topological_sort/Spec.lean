import LeanMode.graph_topological_sort.GraphTopologicalSort
import Mathlib

open Aeneas Aeneas.Std Result
open graph_topological_sort

set_option linter.unusedVariables false

/-!
# Specification: Topological Sort (Kahn's algorithm)

`topological_sort n from_list to_list` returns `ok (some order)` when the
directed graph `(n, from_list, to_list)` is a DAG, where `order` is a valid
topological ordering; and `ok none` when the graph contains a cycle.

The abstract graph is a `Finset (Nat × Nat)` of directed edges, matching the
`graph_dijkstra` spec style.
-/

/-- Directed edge `u → v`. -/
def has_edge (edges : Finset (Nat × Nat)) (u v : Nat) : Prop :=
  (u, v) ∈ edges

/-- A (possibly empty) path from `u` to `v`. -/
inductive has_path (edges : Finset (Nat × Nat)) : Nat → Nat → Prop
  | empty (v : Nat) : has_path edges v v
  | step (u v w : Nat) : has_edge edges u v → has_path edges v w → has_path edges u w

/-- The graph contains a cycle: some vertex `u` lies on a directed cycle. -/
def graph_has_cycle (edges : Finset (Nat × Nat)) : Prop :=
  ∃ u v, has_edge edges u v ∧ has_path edges v u


/-! ## View helpers -/

/-- View a `Vec Std.Usize` as `Fin n → Nat`. -/
def toOrderVec (v : alloc.vec.Vec Std.Usize) (n : Nat) (h : v.length = n) : Fin n → Nat :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    (v[i.val]'h_bound).val

/-- `order` is a permutation of the vertices `0..n-1` with no "back edge":
    for every edge `order i → order j`, `i` comes before `j`. -/
def is_topological_ordering (edges : Finset (Nat × Nat)) (n : Nat)
    (order : Fin n → Nat) : Prop :=
  (∀ i, order i < n) ∧
  (∀ i j, i ≠ j → order i ≠ order j) ∧
  (∀ i j, j.val < i.val → ¬ has_edge edges (order i) (order j))


/-! ## Specification theorems -/

/-- If Kahn's algorithm returns an ordering, the graph is acyclic and the
    ordering is a valid topological ordering. -/
theorem topological_sort_correct_some
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (edges : Finset (Nat × Nat))
    (order : alloc.vec.Vec Std.Usize)
    (h_len : order.length = n.val)
    (h : topological_sort n from_list to_list = ok (some order)) :
    ¬ graph_has_cycle edges ∧
    is_topological_ordering edges n.val (toOrderVec order n.val h_len) := by
  sorry

/-- If Kahn's algorithm returns `none`, the graph contains a cycle. -/
theorem topological_sort_correct_none
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (edges : Finset (Nat × Nat))
    (h : topological_sort n from_list to_list = ok none) :
    graph_has_cycle edges := by
  sorry
