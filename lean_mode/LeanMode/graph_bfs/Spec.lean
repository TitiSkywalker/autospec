import LeanMode.graph_bfs.GraphBfs
import Mathlib

open Aeneas Aeneas.Std Result
open graph_bfs

set_option linter.unusedVariables false

/-!
# Specification: Breadth-First Search (unweighted shortest path)

`bfs n from_list to_list start target` returns `ok (some d)` where `d` is the
shortest distance (number of edges) on a directed path from `start` to
`target`, or `ok none` if `target` is unreachable from `start`.

The abstract graph is a `Finset (Nat × Nat)` of directed edges, matching the
`graph_topological_sort` spec style.
-/

/-- Directed edge `u → v`. -/
def has_edge (edges : Finset (Nat × Nat)) (u v : Nat) : Prop :=
  (u, v) ∈ edges

/-- A path of exactly `d` edges from `u` to `v`. -/
inductive has_path (edges : Finset (Nat × Nat)) : Nat → Nat → Nat → Prop
  | empty (v : Nat) : has_path edges v v 0
  | step (u v w : Nat) (d : Nat) :
      has_edge edges u v → has_path edges v w d → has_path edges u w (d + 1)

/-- `v` is reachable from `start`. -/
def reachable (edges : Finset (Nat × Nat)) (start v : Nat) : Prop :=
  ∃ d, has_path edges start v d

/-- `d` is the shortest distance (edge count) from `start` to `v`. -/
def is_shortest_distance (edges : Finset (Nat × Nat)) (start v d : Nat) : Prop :=
  has_path edges start v d ∧ ∀ e, has_path edges start v e → d ≤ e


/-! ## Specification theorems -/

/-- If BFS returns a distance, it is the shortest distance from `start` to
    `target`. -/
theorem bfs_correct_some
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (start target : Std.Usize)
    (edges : Finset (Nat × Nat))
    (d : Std.Usize)
    (h : bfs n from_list to_list start target = ok (some d)) :
    is_shortest_distance edges start.val target.val d.val := by
  sorry

/-- If BFS returns `none`, `target` is unreachable from `start`. -/
theorem bfs_correct_none
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (start target : Std.Usize)
    (edges : Finset (Nat × Nat))
    (h : bfs n from_list to_list start target = ok none) :
    ¬ reachable edges start.val target.val := by
  sorry
