import LeanMode.array_quick_sort.ArrayQuickSort
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open array_quick_sort
open List

set_option linter.unusedVariables false

/-!
# Specification: Quicksort

-/

/-- Correctness of quicksort: the output is sorted and is a permutation
    of the input. -/
theorem quick_sort_correct (input : Slice Int) (result : alloc.vec.Vec Int)
    (h : quick_sort input = ok result) :
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Int) ∧
    (alloc.vec.Vec.deref result : List Int).Perm (input : List Int) :=
by
  sorry
