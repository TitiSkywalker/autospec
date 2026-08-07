import LeanMode.array_merge_sort.ArrayMergeSort
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open array_merge_sort
open List

set_option linter.unusedVariables false

/-!
# Specification: Merge Sort

-/

/-- Correctness of merge sort: the output is sorted and is a permutation
    of the input. -/
theorem merge_sort_correct (input : Slice Int) (result : alloc.vec.Vec Int)
    (h : merge_sort input = ok result) :
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Int) ∧
    (alloc.vec.Vec.deref result : List Int).Perm (input : List Int) :=
by
  sorry
