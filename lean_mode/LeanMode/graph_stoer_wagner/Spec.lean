import LeanMode.graph_stoer_wagner.GraphStoerWagner
import Mathlib

open Aeneas Aeneas.Std Result
open graph_stoer_wagner

set_option linter.unusedVariables false

/-!
# Specification: Global Minimum Cut (Stoer–Wagner)

`stoer_wagner n weight` returns `ok value` where `value` is the total weight of
a global minimum cut of the undirected weighted graph given by the flat,
row-major, symmetric `n × n` weight matrix `weight` (`weight u v` is the weight
of the undirected edge `u — v`, `0` = no edge).

A *cut* is a non-empty proper vertex subset `S`; its value is the total weight
of the edges crossing `S`.  The theorem states that the returned value is
attained by some cut and that no cut has a smaller value (the "cut of the
phase" lemma is the subtle part of the proof).
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

/-- `S` is a cut: a non-empty proper subset of the vertices. -/
def is_cut {n : Nat} (S : Finset (Fin n)) : Prop :=
  (∃ i : Fin n, i ∈ S) ∧ (∃ i : Fin n, i ∉ S)

/-- The value of a cut: total weight of edges crossing `S` (each crossing edge
    counted once, from the `S` side to the complement). -/
def cut_value {n : Nat} (w : Fin n → Fin n → Int) (S : Finset (Fin n)) : Int :=
  ∑ i : Fin n, ∑ j : Fin n, if i ∈ S ∧ j ∉ S then w i j else 0

/-- `value` is a global minimum cut value: it is attained by some cut, and no
    cut is smaller. -/
def is_min_cut {n : Nat} (w : Fin n → Fin n → Int) (value : Int) : Prop :=
  (∃ S : Finset (Fin n), is_cut S ∧ cut_value w S = value) ∧
  (∀ S : Finset (Fin n), is_cut S → value ≤ cut_value w S)


/-! ## Specification theorems -/

/-- Soundness (optimality): no cut has value smaller than the returned value. -/
theorem stoer_wagner_sound
    (n : Std.Usize) (weight : Slice Int) (value : Int)
    (h_weight : weight.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix weight n.val h_weight i j =
                                 toMatrix weight n.val h_weight j i)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix weight n.val h_weight i j)
    (h_n : 2 ≤ n.val)
    (h : stoer_wagner n weight = ok value) :
    ∀ S : Finset (Fin n.val), is_cut S →
      value ≤ cut_value (toMatrix weight n.val h_weight) S := by
  sorry

/-- Completeness (attainability): the returned value is the value of some cut. -/
theorem stoer_wagner_complete
    (n : Std.Usize) (weight : Slice Int) (value : Int)
    (h_weight : weight.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix weight n.val h_weight i j =
                                 toMatrix weight n.val h_weight j i)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix weight n.val h_weight i j)
    (h_n : 2 ≤ n.val)
    (h : stoer_wagner n weight = ok value) :
    ∃ S : Finset (Fin n.val), is_cut S ∧
      cut_value (toMatrix weight n.val h_weight) S = value := by
  sorry

/-- The returned value is a global minimum cut. -/
theorem stoer_wagner_min_cut
    (n : Std.Usize) (weight : Slice Int) (value : Int)
    (h_weight : weight.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix weight n.val h_weight i j =
                                 toMatrix weight n.val h_weight j i)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix weight n.val h_weight i j)
    (h_n : 2 ≤ n.val)
    (h : stoer_wagner n weight = ok value) :
    is_min_cut (toMatrix weight n.val h_weight) value := by
  sorry
