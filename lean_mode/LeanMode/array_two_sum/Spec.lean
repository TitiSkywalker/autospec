import LeanMode.array_two_sum.ArrayTwoSum
import Mathlib

open Aeneas Aeneas.Std Result
open array_two_sum

set_option linter.unusedVariables false

/-!
# Specification: two-sum via two pointers

Given a sorted slice and a target value, `two_sum nums target` returns `true`
iff there are two distinct indices `i < j` with `nums[i] + nums[j] = target`.
-/

/-- Correctness of `two_sum`: assuming the input is sorted in non-decreasing
    order, the result is `true` exactly when two distinct elements sum to the
    target. -/
theorem two_sum_correct
    (nums : Slice Int) (target : Int) (result : Bool)
    (h_sorted : List.Pairwise (· ≤ ·) (nums : List Int))
    (h : two_sum nums target = ok result) :
    result = true ↔
      ∃ i j, i < j ∧
        i < (nums : List Int).length ∧ j < (nums : List Int).length ∧
        (nums : List Int).getD i 0 + (nums : List Int).getD j 0 = target := by
  sorry
