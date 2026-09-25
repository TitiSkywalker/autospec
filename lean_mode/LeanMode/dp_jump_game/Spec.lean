import LeanMode.dp_jump_game.DpJumpGame
import Mathlib

open Aeneas Aeneas.Std Result
open dp_jump_game

set_option linter.unusedVariables false

/-!
# Specification: Jump Game

`can_jump nums` returns `true` iff the last index is reachable from index 0,
where each position `i` allows jumping to any index `j` with `i < j ≤ i + nums[i]`.
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- `path` is a valid jump path through `nums`: it starts at 0, ends at the last
    index, stays in range, and every consecutive step is a legal jump. -/
def is_valid_jump_path (path : List Nat) (nums : List Nat) : Prop :=
  path.length > 0 ∧
  path.head? = some 0 ∧
  path.getLast? = some (nums.length - 1) ∧
  (∀ i, i < path.length → path.getD i 0 < nums.length) ∧
  (∀ i, i < path.length - 1 →
    let u := path.getD i 0
    let v := path.getD (i + 1) 0
    u < v ∧ v ≤ u + nums.getD u 0)

/-- Correctness of `can_jump`: the result is `true` exactly when a valid jump
    path to the last index exists. -/
theorem can_jump_correct (nums : Slice Std.Usize) (result : Bool)
    (h_pos : 0 < nums.length)
    (h : can_jump nums = ok result) :
    (result = true) ↔ ∃ (path : List Nat), is_valid_jump_path path (toNatList nums) :=
by
  sorry
