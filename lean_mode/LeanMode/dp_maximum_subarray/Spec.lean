import LeanMode.dp_maximum_subarray.DpMaximumSubarray
import Mathlib

open Aeneas Aeneas.Std Result
open dp_maximum_subarray

set_option linter.unusedVariables false

/-!
# Specification: Maximum Subarray Sum (Kadane's algorithm)

`max_subarray_sum nums` returns the maximum sum of any contiguous subarray of
`nums`.
-/

/-- Convert `Slice Int` to `List Int`. -/
def toIntList (s : Slice Int) : List Int :=
  (s : List Int)

/-- Sum of the contiguous subarray `seq[i..j)` (indices `i .. j-1`). -/
def spec_sum (seq : List Int) (i j : Nat) : Int :=
  ((seq.drop i).take (j - i)).foldl (fun acc x => acc + x) 0

/-- Correctness of `max_subarray_sum`:
    (1) Upper bound: every contiguous subarray sums to at most `result`.
    (2) Attainment: some contiguous subarray sums to exactly `result`. -/
theorem max_subarray_sum_correct (nums : Slice Int) (result : Int)
    (h_pos : 0 < nums.length)
    (h : max_subarray_sum nums = ok result) :
    (∀ i j : Nat, i ≤ j → j ≤ nums.length → spec_sum (toIntList nums) i j ≤ result) ∧
    (∃ i j : Nat, i ≤ j ∧ j ≤ nums.length ∧ spec_sum (toIntList nums) i j = result) :=
by
  sorry
