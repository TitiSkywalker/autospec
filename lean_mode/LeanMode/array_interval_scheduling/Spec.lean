import LeanMode.array_interval_scheduling.ArrayIntervalScheduling
import Mathlib

open Aeneas Aeneas.Std Result
open array_interval_scheduling

set_option linter.unusedVariables false

/-!
# Specification: maximum number of pairwise non-overlapping intervals.

`interval_scheduling starts ends` returns the maximum size of a subset of
intervals such that no two chosen intervals overlap.
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (fun x => x.val)

/-- Number of intervals selected by the boolean mask `selected`. -/
def count_selected (selected : List Bool) : Nat :=
  selected.foldl (fun acc b => if b then acc + 1 else acc) 0

/-- The intervals selected by `selected` are pairwise non-overlapping. -/
def no_overlap (selected : List Bool) (starts ends : List Nat) : Prop :=
  ∀ i j, i < j →
    i < selected.length → j < selected.length →
    selected.getD i false = true → selected.getD j false = true →
    ends.getD i 0 ≤ starts.getD j 0 ∨ ends.getD j 0 ≤ starts.getD i 0

/-- Correctness of the greedy interval scheduler:
    (1) Optimality: every non-overlapping selection has at most `result` intervals.
    (2) Attainability: some non-overlapping selection has exactly `result` intervals. -/
theorem interval_scheduling_correct
    (starts ends : Slice Std.Usize) (result : Std.Usize)
    (h_len : starts.length = ends.length)
    (h_pos : ∀ i, i < starts.length →
       (toNatList starts).getD i 0 < (toNatList ends).getD i 0)
    (h : interval_scheduling starts ends = ok result) :
    (∀ selected : List Bool,
       selected.length = starts.length →
       no_overlap selected (toNatList starts) (toNatList ends) →
       count_selected selected ≤ result.val) ∧
    (∃ selected : List Bool,
       selected.length = starts.length ∧
       no_overlap selected (toNatList starts) (toNatList ends) ∧
       count_selected selected = result.val) := by
  sorry
