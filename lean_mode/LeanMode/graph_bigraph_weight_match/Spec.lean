import LeanMode.graph_bigraph_weight_match.GraphBigraphWeightMatch
import Mathlib

open Aeneas Aeneas.Std Result
open graph_bigraph_weight_match

set_option linter.unusedVariables false

/-!
# Specification: Maximum-Weight Bipartite Matching (Kuhn–Munkres)

`kuhn_munkres n w` returns `ok (weight, matched)` where `w` is a flat, row-major
`n × n` weight matrix and `matched` is a flat `n × n` 0/1 matrix marking a
*perfect* matching (every left vertex matched to a distinct right vertex).
`weight` is its total weight.

The theorems assert that the returned matrix is a perfect matching, that
`weight` equals its total weight, and that no perfect matching has larger total
weight (the optimality proof goes through feasible vertex labels and the
equality subgraph — the duality insight).
-/

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) as `Fin n → Fin n → Int`. -/
def toMatrix (s : Slice Int) (n : Nat) (h : s.length = n * n) : Fin n → Fin n → Int :=
  λ i j =>
    have h_bound : i.val * n + j.val < s.length := by
      rw [h]
      have hi : i.val < n := i.is_lt
      have hj : j.val < n := j.is_lt
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left hj (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt hi)
    s[i.val * n + j.val]'h_bound

/-- `matched` is a perfect matching: 0/1 entries, exactly one matched edge per
    left vertex and per right vertex. -/
def is_perfect_matching {n : Nat} (matched : Fin n → Fin n → Int) : Prop :=
  (∀ u v : Fin n, matched u v = 0 ∨ matched u v = 1) ∧
  (∀ u : Fin n, (∑ v : Fin n, matched u v) = 1) ∧
  (∀ v : Fin n, (∑ u : Fin n, matched u v) = 1)

/-- The total weight of a matching. -/
def matching_weight {n : Nat} (w matched : Fin n → Fin n → Int) : Int :=
  ∑ u : Fin n, ∑ v : Fin n, matched u v * w u v

/-- `matched` is a maximum-weight perfect matching: it is perfect, and no
    perfect matching has larger total weight. -/
def is_max_weight {n : Nat} (w matched : Fin n → Fin n → Int) : Prop :=
  is_perfect_matching matched ∧
  ∀ other : Fin n → Fin n → Int,
    is_perfect_matching other → matching_weight w other ≤ matching_weight w matched


/-! ## Specification theorems -/

/-- `kuhn_munkres` returns a matching matrix of length `n * n`. -/
theorem kuhn_munkres_length
    (n : Std.Usize) (w : Slice Int) (weight : Int) (matched : alloc.vec.Vec Int)
    (h : kuhn_munkres n w = ok (weight, matched)) :
    (alloc.vec.Vec.deref matched).length = n.val * n.val := by
  sorry

/-- Soundness: the returned matrix is a perfect matching. -/
theorem kuhn_munkres_is_perfect
    (n : Std.Usize) (w : Slice Int) (weight : Int) (matched : alloc.vec.Vec Int)
    (h_w : w.length = n.val * n.val)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : kuhn_munkres n w = ok (weight, matched)) :
    is_perfect_matching (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- The returned `weight` equals the total weight of the returned matching. -/
theorem kuhn_munkres_weight
    (n : Std.Usize) (w : Slice Int) (weight : Int) (matched : alloc.vec.Vec Int)
    (h_w : w.length = n.val * n.val)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : kuhn_munkres n w = ok (weight, matched)) :
    weight = matching_weight (toMatrix w n.val h_w)
                             (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- Optimality: the returned matching has maximum total weight. -/
theorem kuhn_munkres_maximal
    (n : Std.Usize) (w : Slice Int) (weight : Int) (matched : alloc.vec.Vec Int)
    (h_w : w.length = n.val * n.val)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : kuhn_munkres n w = ok (weight, matched)) :
    is_max_weight (toMatrix w n.val h_w)
                  (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry
