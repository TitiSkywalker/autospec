import LeanMode.graph_bipartite_check.GraphBipartiteCheck
import Mathlib

open Aeneas Aeneas.Std Result
open graph_bipartite_check

set_option linter.unusedVariables false

/-!
# Specification: Bipartite Check (BFS 2-coloring)

`is_bipartite n from_list to_list` returns `ok (some colors)` when the
undirected graph `(n, from_list, to_list)` is 2-colorable, where `colors` is a
valid 2-coloring (`0` / `1` with every edge connecting different colors); and
`ok none` when the graph is not bipartite.

Each undirected edge `{u, v}` appears once in the edge list, so the abstract
graph `Finset (Nat × Nat)` is read symmetrically.
-/

/-- Undirected edge `{u, v}`. -/
def has_edge (edges : Finset (Nat × Nat)) (u v : Nat) : Prop :=
  (u, v) ∈ edges ∨ (v, u) ∈ edges

/-- `colors` is a valid 2-coloring: values in `{0,1}`, adjacent vertices differ. -/
def is_valid_coloring (edges : Finset (Nat × Nat)) (n : Nat)
    (colors : Fin n → Nat) : Prop :=
  (∀ i, colors i < 2) ∧
  (∀ (u v : Fin n), has_edge edges u.val v.val → colors u ≠ colors v)

/-- The graph is bipartite (admits some valid 2-coloring). -/
def is_bipartite (edges : Finset (Nat × Nat)) (n : Nat) : Prop :=
  ∃ colors : Fin n → Nat, is_valid_coloring edges n colors


/-! ## View helpers -/

/-- View a `Vec Std.Usize` as `Fin n → Nat`. -/
def toColorVec (v : alloc.vec.Vec Std.Usize) (n : Nat) (h : v.length = n) : Fin n → Nat :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    (v[i.val]'h_bound).val


/-! ## Specification theorems -/

/-- If BFS 2-coloring succeeds, the returned coloring is valid. -/
theorem is_bipartite_correct_some
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (edges : Finset (Nat × Nat))
    (colors : alloc.vec.Vec Std.Usize)
    (h_len : colors.length = n.val)
    (h : is_bipartite n from_list to_list = ok (some colors)) :
    is_valid_coloring edges n.val (toColorVec colors n.val h_len) := by
  sorry

/-- If BFS 2-coloring fails, the graph is not bipartite. -/
theorem is_bipartite_correct_none
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (edges : Finset (Nat × Nat))
    (h : is_bipartite n from_list to_list = ok none) :
    ¬ is_bipartite edges n.val := by
  sorry
