import LeanMode.dp_knapsack.DpKnapsack
import Mathlib

open Aeneas Aeneas.Std Result
open dp_knapsack

set_option linter.unusedVariables false

/-!
# Specification: 0/1 Knapsack

`knapsack weights values capacity` returns the maximum total value achievable
by selecting a subset of items whose total weight does not exceed the capacity.
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- Total weight of the items selected by `selected`. -/
def total_weight (selected : List Bool) (weights : List Nat) : Nat :=
  match selected, weights with
  | [], _ => 0
  | _, [] => 0
  | b :: bs, w :: ws => (if b then w else 0) + total_weight bs ws

/-- Total value of the items selected by `selected`. -/
def total_value (selected : List Bool) (values : List Nat) : Nat :=
  match selected, values with
  | [], _ => 0
  | _, [] => 0
  | b :: bs, v :: vs => (if b then v else 0) + total_value bs vs

/-- A selection is valid iff it covers all items and total weight ≤ capacity. -/
def is_valid_strategy (selected : List Bool) (weights : List Nat) (capacity : Nat) : Prop :=
  selected.length = weights.length ∧
  total_weight selected weights ≤ capacity

/-- Correctness of the iterative DP knapsack:
    (1) Optimality: any feasible selection's total value ≤ result.
    (2) Attainability: there exists a feasible selection achieving exactly result. -/
theorem knapsack_correct
    (weights values : Slice Std.Usize) (capacity : Std.Usize) (result : Std.Usize)
    (h_len : weights.length = values.length)
    (h : knapsack weights values capacity = ok result) :
    (∀ (selected : List Bool),
       is_valid_strategy selected (toNatList weights) capacity.val →
       total_value selected (toNatList values) ≤ result.val) ∧
    (∃ (selected : List Bool),
       is_valid_strategy selected (toNatList weights) capacity.val ∧
       total_value selected (toNatList values) = result.val) :=
by
  sorry