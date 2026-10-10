import LeanMode.graph_topological_sort.GraphTopologicalSort
import Mathlib

open Aeneas Aeneas.Std Result
open graph_topological_sort

set_option linter.unusedVariables false

/-!
# Specification: Topological Sort (Kahn's algorithm)

`topological_sort n from_list to_list` returns `ok (some order)` when the
directed graph is a DAG, where `order` is a valid topological ordering; and
`ok none` when the graph contains a cycle.  The directed graph is read
directly off the concrete CSR edge list.
-/

/-- A directed graph given as parallel `from_list` / `to_list` slices. -/
structure CsrGraph where
  from_list : Slice Std.Usize
  to_list : Slice Std.Usize
  h_len : from_list.length = to_list.length

/-- View an edge-list slice of length `m` as `Fin m → Nat`. -/
def toEdgeVec (s : Slice Std.Usize) (m : Nat) (h : s.length = m) : Fin m → Nat :=
  λ k =>
    have hb : k.val < s.length := by rw [h]; exact k.is_lt
    (s[k.val]'hb).val

/-- Directed edge `u → v` in the concrete edge list. -/
def edge_in (g : CsrGraph) (u v : Nat) : Prop :=
  ∃ k : Fin g.from_list.length,
    toEdgeVec g.from_list g.from_list.length rfl k = u ∧
    toEdgeVec g.to_list g.from_list.length g.h_len.symm k = v

/-- A (possibly empty) path from `u` to `v`. -/
inductive has_path (g : CsrGraph) : Nat → Nat → Prop
  | empty (v : Nat) : has_path g v v
  | step (u v w : Nat) : edge_in g u v → has_path g v w → has_path g u w

/-- The graph contains a cycle: some vertex `u` lies on a directed cycle. -/
def graph_has_cycle (g : CsrGraph) : Prop :=
  ∃ u v, edge_in g u v ∧ has_path g v u


/-! ## View helpers -/

/-- View a `Vec Std.Usize` as `Fin n → Nat`. -/
def toOrderVec (v : alloc.vec.Vec Std.Usize) (n : Nat) (h : v.length = n) : Fin n → Nat :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    (v[i.val]'h_bound).val

/-- `order` is a permutation of the vertices `0..n-1` with no "back edge":
    for every edge `order i → order j`, `i` comes before `j`. -/
def is_topological_ordering (g : CsrGraph) (n : Nat) (order : Fin n → Nat) : Prop :=
  (∀ i, order i < n) ∧
  (∀ i j, i ≠ j → order i ≠ order j) ∧
  (∀ i j, j.val < i.val → ¬ edge_in g (order i) (order j))


/-! ## Specification theorems -/

/-- If Kahn's algorithm returns an ordering, the graph is acyclic and the
    ordering is a valid topological ordering. -/
theorem topological_sort_correct_some
    (n : Std.Usize)
    (g : CsrGraph)
    (order : alloc.vec.Vec Std.Usize)
    (h_len : order.length = n.val)
    (h : topological_sort n g.from_list g.to_list = ok (some order)) :
    ¬ graph_has_cycle g ∧
    is_topological_ordering g n.val (toOrderVec order n.val h_len) := by
  sorry

/-- If Kahn's algorithm returns `none`, the graph contains a cycle. -/
theorem topological_sort_correct_none
    (n : Std.Usize)
    (g : CsrGraph)
    (h : topological_sort n g.from_list g.to_list = ok none) :
    graph_has_cycle g := by
  sorry
