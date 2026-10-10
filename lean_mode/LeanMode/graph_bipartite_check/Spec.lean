import LeanMode.graph_bipartite_check.GraphBipartiteCheck
import Mathlib

open Aeneas Aeneas.Std Result
open graph_bipartite_check

set_option linter.unusedVariables false

/-!
# Specification: Bipartite Check (BFS 2-coloring)

`is_bipartite n from_list to_list` returns `ok (some colors)` when the
undirected graph is 2-colorable, where `colors` is a valid 2-coloring; and
`ok none` when the graph is not bipartite.  The undirected graph is read
directly off the concrete CSR edge list (each undirected edge `{u, v}`
appears once).
-/

/-- An undirected graph given as parallel `from_list` / `to_list` slices. -/
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

/-- Undirected edge `{u, v}` (the list is read symmetrically). -/
def has_edge (g : CsrGraph) (u v : Nat) : Prop :=
  edge_in g u v ∨ edge_in g v u

/-- `colors` is a valid 2-coloring: values in `{0,1}`, adjacent vertices differ. -/
def is_valid_coloring (g : CsrGraph) (n : Nat) (colors : Fin n → Nat) : Prop :=
  (∀ i, colors i < 2) ∧
  (∀ (u v : Fin n), has_edge g u.val v.val → colors u ≠ colors v)

/-- The graph is bipartite (admits some valid 2-coloring). -/
def is_bipartite (g : CsrGraph) (n : Nat) : Prop :=
  ∃ colors : Fin n → Nat, is_valid_coloring g n colors


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
    (g : CsrGraph)
    (colors : alloc.vec.Vec Std.Usize)
    (h_len : colors.length = n.val)
    (h : is_bipartite n g.from_list g.to_list = ok (some colors)) :
    is_valid_coloring g n.val (toColorVec colors n.val h_len) := by
  sorry

/-- If BFS 2-coloring fails, the graph is not bipartite. -/
theorem is_bipartite_correct_none
    (n : Std.Usize)
    (g : CsrGraph)
    (h : is_bipartite n g.from_list g.to_list = ok none) :
    ¬ is_bipartite g n.val := by
  sorry
