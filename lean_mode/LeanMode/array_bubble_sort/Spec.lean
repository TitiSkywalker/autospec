import LeanMode.array_bubble_sort.ArrayBubbleSort
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open array_bubble_sort
open List

set_option linter.unusedVariables false

/-!
# Specification: Bubble Sort

-/

/-- Correctness of bubble sort: the output is sorted and is a permutation
    of the input. -/
theorem bubble_sort_correct (input : Slice Int) (result : alloc.vec.Vec Int)
    (h : bubble_sort input = ok result) :
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Int) ∧
    (alloc.vec.Vec.deref result : List Int).Perm (input : List Int) :=
by
  sorry
