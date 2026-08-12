import LeanMode.array_binary_search.ArrayBinarySearch
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open array_binary_search
open List

set_option linter.unusedVariables false

/-!
# Specification: Binary Search Lower Bound

Given a sorted input slice and a target value, `binary_search_lower_bound` returns
the first index `i` such that `input[i] >= target`, or `input.len()` if no such index exists.
-/

/-- Correctness of `binary_search_lower_bound`:
    If the input slice is sorted in non-decreasing order and the function
    returns `ok result`, then:
    1. `result ≤ input.length` (the index is within bounds)
    2. All elements before `result` are strictly less than `target`
    3. All elements at or after `result` are greater than or equal to `target`
-/
theorem binary_search_lower_bound_correct
    (input : Slice Int) (target : Int) (result : Std.Usize)
    (h_sorted : Pairwise (· ≤ ·) (input : List Int))
    (h : binary_search_lower_bound input target = ok result) :
    (result.val : Nat) ≤ (input : List Int).length ∧
    (∀ i : Nat, i < (result.val : Nat) → ((input : List Int).getD i 0) < target) ∧
    (∀ i : Nat, (result.val : Nat) ≤ i → i < (input : List Int).length → ((input : List Int).getD i 0) ≥ target) :=
by
  sorry
