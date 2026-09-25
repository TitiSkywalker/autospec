import LeanMode.dp_longest_common_subsequence.DpLongestCommonSubsequence
import Mathlib

open Aeneas Aeneas.Std Result
open dp_longest_common_subsequence

set_option linter.unusedVariables false

/-!
# Specification: Longest Common Subsequence

`lcs a b` returns the length of the longest common subsequence of `a` and `b`.
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- `sub` is a subsequence of `s` (obtained by deleting zero or more elements). -/
def is_subsequence (sub : List Nat) (s : List Nat) : Prop :=
  match s with
  | [] => sub = []
  | y :: ys =>
    match sub with
    | [] => True
    | x :: xs => (x = y ∧ is_subsequence xs ys) ∨ is_subsequence (x :: xs) ys

/-- Correctness of `lcs`:
    (1) Optimality: the length of any common subsequence is at most `result`.
    (2) Attainability: some common subsequence has length exactly `result`. -/
theorem lcs_correct (a b : Slice Std.Usize) (result : Std.Usize)
    (h : lcs a b = ok result) :
    (∀ (sub : List Nat),
       is_subsequence sub (toNatList a) →
       is_subsequence sub (toNatList b) →
       sub.length ≤ result.val) ∧
    (∃ (sub : List Nat),
       is_subsequence sub (toNatList a) ∧
       is_subsequence sub (toNatList b) ∧
       sub.length = result.val) :=
by
  sorry
