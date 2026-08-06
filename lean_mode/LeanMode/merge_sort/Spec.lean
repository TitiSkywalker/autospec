import LeanMode.merge_sort.MergeSort
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open merge_sort
open List

set_option linter.unusedVariables false

/-!
# Specification: Merge Sort

-/

/-- Correctness of merge sort: the output is sorted and is a permutation
    of the input. -/
theorem merge_sort_correct (input : Slice Rat) (result : alloc.vec.Vec Rat)
    (h : merge_sort input = ok result) :
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Rat) ∧
    (alloc.vec.Vec.deref result : List Rat).Perm (input : List Rat) :=
by
  sorry
