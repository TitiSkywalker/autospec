import LeanMode.dp_lis.DpLis
import Mathlib

open Aeneas Aeneas.Std Result
open dp_lis

set_option linter.unusedVariables false

/-!
# Specification: Longest Increasing Subsequence

`lis nums` returns the length of the longest strictly increasing subsequence
of `nums`.
-/

/-- Convert `Slice Int` to `List Int`. -/
def toIntList (s : Slice Int) : List Int :=
  (s : List Int)

/-- `sub` is a subsequence of `s` (obtained by deleting zero or more elements). -/
def is_subsequence (sub : List Int) (s : List Int) : Prop :=
  match s with
  | [] => sub = []
  | y :: ys =>
    match sub with
    | [] => True
    | x :: xs => (x = y ∧ is_subsequence xs ys) ∨ is_subsequence (x :: xs) ys

/-- `sub` is strictly increasing. -/
def is_increasing (sub : List Int) : Prop :=
  List.Pairwise (· < ·) sub

/-- Correctness of `lis`:
    (1) Optimality: every strictly increasing subsequence has length at most `result`.
    (2) Attainability: some strictly increasing subsequence has length exactly `result`. -/
theorem lis_correct (nums : Slice Int) (result : Std.Usize)
    (h : lis nums = ok result) :
    (∀ (sub : List Int),
       is_subsequence sub (toIntList nums) →
       is_increasing sub →
       sub.length ≤ result.val) ∧
    (∃ (sub : List Int),
       is_subsequence sub (toIntList nums) ∧
       is_increasing sub ∧
       sub.length = result.val) :=
by
  sorry
