import LeanMode.bubble_sort.BubbleSort
import Mathlib.Data.List.Sort

open Aeneas Aeneas.Std Result
open bubble_sort
open List

set_option linter.unusedVariables false

/-!
# Specification: Bubble Sort

-/

/-- Correctness of bubble sort: the output is sorted and is a permutation
    of the input. -/
theorem bubble_sort_correct (input : Slice Rat) (result : alloc.vec.Vec Rat)
    (h : bubble_sort input = ok result) :
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Rat) ∧
    (alloc.vec.Vec.deref result : List Rat).Perm (input : List Rat) :=
by
  sorry
