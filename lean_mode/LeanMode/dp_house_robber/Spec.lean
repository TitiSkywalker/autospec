import LeanMode.dp_house_robber.DpHouseRobber
import Mathlib

open Aeneas Aeneas.Std Result
open dp_house_robber

set_option linter.unusedVariables false

/-!
# Specification: House Robber

`rob nums` returns the maximum total loot obtainable without robbing adjacent
houses.
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- A list of chosen house indices is a valid (non-adjacent) robbery plan for
    `len` houses: every index is in range and consecutive indices differ by at
    least 2. -/
def is_valid_robbery (houses : List Nat) (len : Nat) : Prop :=
  (∀ i, i < houses.length → houses.getD i 0 < len) ∧
  (∀ i, i + 1 < houses.length →
    houses.getD (i + 1) 0 ≥ houses.getD i 0 + 2)

/-- Total loot of a robbery plan against `values`. -/
def total_loot (houses : List Nat) (values : List Nat) : Nat :=
  houses.foldl (fun acc i => acc + values.getD i 0) 0

/-- Correctness of `rob`:
    (1) Optimality: any valid non-adjacent robbery plan yields at most `result`.
    (2) Attainability: some valid plan achieves exactly `result`. -/
theorem rob_correct (nums : Slice Std.Usize) (result : Std.Usize)
    (h : rob nums = ok result) :
    (∀ (houses : List Nat),
       is_valid_robbery houses nums.length →
       total_loot houses (toNatList nums) ≤ result.val) ∧
    (∃ (houses : List Nat),
       is_valid_robbery houses nums.length ∧
       total_loot houses (toNatList nums) = result.val) :=
by
  sorry
