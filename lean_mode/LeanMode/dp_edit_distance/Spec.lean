import LeanMode.dp_edit_distance.DpEditDistance
import Mathlib

open Aeneas Aeneas.Std Result
open dp_edit_distance

set_option linter.unusedVariables false

/-!
# Specification: Edit (Levenshtein) distance

`edit_distance a b` returns the minimum number of insertions, deletions, or
substitutions needed to transform `a` into `b`.
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (fun x => x.val)

/-- The Levenshtein distance as a well-founded recursion on the sum of the two
    list lengths. -/
def edit_spec : List Nat → List Nat → Nat
  | [], b => b.length
  | a, [] => a.length
  | x :: xs, y :: ys =>
      min (min (edit_spec xs (y :: ys) + 1) (edit_spec (x :: xs) ys + 1))
        (edit_spec xs ys + (if x = y then 0 else 1))
termination_by a b => a.length + b.length
decreasing_by
  all_goals simp_wf
  omega

/-- Correctness of `edit_distance`: the result equals the Levenshtein distance. -/
theorem edit_distance_correct (a b : Slice Std.Usize) (result : Std.Usize)
    (h : edit_distance a b = ok result) :
    result.val = edit_spec (toNatList a) (toNatList b) := by
  sorry
