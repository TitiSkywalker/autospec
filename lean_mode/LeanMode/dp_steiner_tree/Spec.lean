import LeanMode.dp_steiner_tree.DpSteinerTree
import Mathlib

open Aeneas Aeneas.Std Result
open dp_steiner_tree

set_option linter.unusedVariables false

/-!
# Specification: minimum Steiner tree (Dreyfus–Wagner)

`steiner_tree n w terminals` runs the Dreyfus–Wagner subset DP over the flat
row-major weight matrix `w` of an undirected complete graph and returns the
weight of a cheapest tree connecting every terminal.

The specification below characterises that weight *declaratively* — as the
minimum, over all edge sets that connect the terminals, of the total edge
weight — with no mention of bitmasks, of the DP table, or of the order in which
the states are enumerated.
-/

namespace SteinerTree

/-- View a flat `usize` slice as a `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- The weight function of the complete graph on `n` vertices, read off the flat
row-major `n × n` matrix `w`: `weightFn w n i j` is the entry `i * n + j`, i.e.
the weight of the (ordered) pair `i → j`. -/
def weightFn (w : Slice Std.Usize) (n : Nat) : Fin n → Fin n → Nat :=
  fun i j => (toNatList w).getD (i.val * n + j.val) 0

/-- The terminal set as a `Finset` of vertices, given the range proof. -/
def termSet {n : Nat} (l : List Std.Usize) (h : ∀ t ∈ l, t.val < n) : Finset (Fin n) :=
  (l.attach.map (fun t => (⟨t.1.val, h t.1 t.2⟩ : Fin n))).toFinset

/-- The edge set `F` makes `u` and `v` adjacent: either ordered pair is in `F`
(the graph is undirected, so an edge may be recorded in either direction). -/
def Edge {n : Nat} (F : Finset (Fin n × Fin n)) (u v : Fin n) : Prop :=
  (u, v) ∈ F ∨ (v, u) ∈ F

/-- `v` is reachable from `u` along the edges of `F`. -/
inductive Reach {n : Nat} (F : Finset (Fin n × Fin n)) : Fin n → Fin n → Prop
  | refl (v : Fin n) : Reach F v v
  | step {u v x : Fin n} : Edge F u v → Reach F v x → Reach F u x

/-- `F` connects every pair of terminals. -/
def IsConnecting {n : Nat} (F : Finset (Fin n × Fin n)) (T : Finset (Fin n)) : Prop :=
  ∀ u ∈ T, ∀ v ∈ T, Reach F u v

/-- Total weight of an edge set: the sum of the weights of its (ordered) pairs. -/
def edgeCost {n : Nat} (w : Fin n → Fin n → Nat) (F : Finset (Fin n × Fin n)) : Nat :=
  F.sum (fun e => w e.1 e.2)

/-- The set of weights of the edge sets connecting `T`. -/
noncomputable def connectingCosts (n : Nat) (w : Fin n → Fin n → Nat)
    (T : Finset (Fin n)) : Finset Nat := by
  classical
  exact ((Finset.univ : Finset (Fin n × Fin n)).powerset.filter
    (fun F => IsConnecting F T)).image (edgeCost w)

/-- The set of connecting weights is non-empty: the complete edge set connects
everything. -/
theorem connectingCosts_nonempty (n : Nat) (w : Fin n → Fin n → Nat)
    (T : Finset (Fin n)) : (connectingCosts n w T).Nonempty := by
  sorry

/-- The cost of a cheapest tree connecting all terminals, i.e. the answer to the
Steiner-tree problem. -/
noncomputable def minSteinerCost (n : Nat) (w : Fin n → Fin n → Nat)
    (T : Finset (Fin n)) : Nat :=
  (connectingCosts n w T).min' (connectingCosts_nonempty n w T)

end SteinerTree

open SteinerTree

/-- Correctness of the Dreyfus–Wagner subset DP: whenever
`steiner_tree n w terminals` returns `c`, the value `c` is the weight of a
cheapest edge set connecting every terminal, where the weight of `i → j` is the
entry `i * n + j` of the flat matrix `w`.

The hypotheses say that `w` really is the weight matrix of an undirected graph:
it is `n × n` (`h_len`) and symmetric (`h_sym`); every terminal is a vertex
(`h_term`). -/
theorem steiner_tree_correct (n : Std.Usize) (w : Slice Std.Usize)
    (terminals : Slice Std.Usize) (c : Std.Usize)
    (h_len : w.length = n.val * n.val)
    (h_term : ∀ t ∈ (terminals : List Std.Usize), t.val < n.val)
    (h_sym : ∀ i j : Fin n.val, weightFn w n.val i j = weightFn w n.val j i)
    (hres : steiner_tree n w terminals = ok c) :
    c.val = minSteinerCost n.val (weightFn w n.val)
      (termSet (terminals : List Std.Usize) h_term) := by
  sorry
