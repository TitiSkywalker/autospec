import LeanMode.quick_sort.QuickSort
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open quick_sort
open List

set_option linter.unusedVariables false

/-!
# Specification: Quicksort

-/

/-- Correctness of quicksort: the output is sorted and is a permutation
    of the input. -/
theorem quick_sort_correct (input : Slice Rat) (result : alloc.vec.Vec Rat)
    (h : quick_sort input = ok result) :
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Rat) ∧
    (alloc.vec.Vec.deref result : List Rat).Perm (input : List Rat) :=
by
  sorry
