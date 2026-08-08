import LeanMode.array_selection_sort.ArraySelectionSort
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open array_selection_sort
open List

set_option linter.unusedVariables false

/-!
# Specification: Selection Sort

-/

/-- Correctness of selection sort: the output is sorted and is a permutation
    of the input. -/
theorem selection_sort_correct (input : Slice Int) (result : alloc.vec.Vec Int)
    (h : selection_sort input = ok result) :
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Int) ∧
    (alloc.vec.Vec.deref result : List Int).Perm (input : List Int) :=
by
  sorry
