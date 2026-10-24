import LeanMode.array_median_of_medians.ArrayMedianOfMedians
import Mathlib

open Aeneas Aeneas.Std Result
open array_median_of_medians

set_option linter.unusedVariables false

/-!
# Specification: Median of Medians (linear-time selection)

`select_kth input k` returns the k-th smallest element of `input` (0-indexed).
-/

/-- Number of elements of `l` that are `≤ r`. -/
def count_le (l : List Int) (r : Int) : Nat :=
  (l.filter (fun x => decide (x ≤ r))).length

/-- Number of elements of `l` that are `≥ r`. -/
def count_ge (l : List Int) (r : Int) : Nat :=
  (l.filter (fun x => decide (x ≥ r))).length

/-- Correctness of `select_kth`: the result is a member of the input, at least
    `k + 1` elements are `≤` it, and at least `n - k` elements are `≥` it —
    i.e. it is the k-th order statistic (0-indexed). -/
theorem select_kth_correct
    (input : Slice Int) (k : Std.Usize) (result : Int)
    (h_k : k.val < (input : List Int).length)
    (h : select_kth input k = ok result) :
    result ∈ (input : List Int) ∧
      k.val + 1 ≤ count_le (input : List Int) result ∧
      (input : List Int).length - k.val ≤ count_ge (input : List Int) result := by
  sorry
