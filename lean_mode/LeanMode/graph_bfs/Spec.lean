import LeanMode.graph_bfs.GraphBfs
import Mathlib

open Aeneas Aeneas.Std Result
open graph_bfs

set_option linter.unusedVariables false

/-!
# Specification: Breadth-First Search (unweighted shortest path)

`bfs n from_list to_list start target` returns `ok (some d)` where `d` is the
shortest distance (number of edges) on a directed path from `start` to
`target`, or `ok none` if `target` is unreachable.  The directed graph is read
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

/-- A path of exactly `d` edges from `u` to `v`. -/
inductive has_path (g : CsrGraph) : Nat → Nat → Nat → Prop
  | empty (v : Nat) : has_path g v v 0
  | step (u v w : Nat) (d : Nat) :
      edge_in g u v → has_path g v w d → has_path g u w (d + 1)

/-- `v` is reachable from `start`. -/
def reachable (g : CsrGraph) (start v : Nat) : Prop :=
  ∃ d, has_path g start v d

/-- `d` is the shortest distance (edge count) from `start` to `v`. -/
def is_shortest_distance (g : CsrGraph) (start v d : Nat) : Prop :=
  has_path g start v d ∧ ∀ e, has_path g start v e → d ≤ e

/-! ## Specification theorems -/

/-- If BFS returns a distance, it is the shortest distance from `start` to
    `target`. -/
theorem bfs_correct_some
    (n : Std.Usize) (g : CsrGraph) (start target : Std.Usize) (d : Std.Usize)
    (h : bfs n g.from_list g.to_list start target = ok (some d)) :
    is_shortest_distance g start.val target.val d.val := by
  sorry

/-- If BFS returns `none`, `target` is unreachable from `start`. -/
theorem bfs_correct_none
    (n : Std.Usize) (g : CsrGraph) (start target : Std.Usize)
    (h : bfs n g.from_list g.to_list start target = ok none) :
    ¬ reachable g start.val target.val := by
  sorry
