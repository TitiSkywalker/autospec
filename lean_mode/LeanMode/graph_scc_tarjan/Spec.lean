import LeanMode.graph_scc_tarjan.GraphSccTarjan
import Mathlib

open Aeneas Aeneas.Std Result
open graph_scc_tarjan

set_option linter.unusedVariables false

/-!
# Specification: Tarjan's Strongly Connected Components

`scc_tarjan n from_list to_list` returns `ok comp` where `comp[v]` is a dense
component id.  The directed graph is read directly off the concrete CSR edge
list (the `CsrGraph` structure): `u → v` is an edge exactly when some position
`k` has `from_list[k] = u` and `to_list[k] = v`.
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

/-- `u` and `v` are mutually reachable (in the same SCC). -/
def strongly_connected (g : CsrGraph) (u v : Nat) : Prop :=
  has_path g u v ∧ has_path g v u

/-- View the output `Vec Std.Usize` as `Fin n → Nat`. -/
def toCompVec (v : alloc.vec.Vec Std.Usize) (n : Nat) (h : v.length = n) : Fin n → Nat :=
  λ i =>
    have hb : i.val < v.length := by rw [h]; exact i.is_lt
    (v[i.val]'hb).val

/-! ## Specification theorem -/

/-- Correctness: two vertices share a component id exactly when they are
    mutually reachable in the input graph. -/
theorem scc_tarjan_correct
    (n : Std.Usize) (g : CsrGraph)
    (comp : alloc.vec.Vec Std.Usize) (h_len : comp.length = n.val)
    (h : scc_tarjan n g.from_list g.to_list = ok comp) :
    ∀ (u v : Fin n.val),
      toCompVec comp n.val h_len u = toCompVec comp n.val h_len v ↔
        strongly_connected g u.val v.val := by
  sorry
