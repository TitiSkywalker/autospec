import LeanMode.dp_coin_change.DpCoinChange
import Mathlib

open Aeneas Aeneas.Std Result
open dp_coin_change

set_option linter.unusedVariables false

/-!
# Specification: Coin Change

`coin_change coins amount` returns the minimum number of coins needed to make
up `amount`.  Asserts that a valid combination exists; the `ok` case guarantees
the result is optimal.
-/

/-- Convert `Slice Std.Usize` to `List Nat` for use in the spec. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- Total monetary value: `Σ counts[i] * coins[i]`. -/
def total_amount (counts : List Nat) (coins : List Nat) : Nat :=
  (counts.zip coins).foldl (fun acc pair => acc + pair.1 * pair.2) 0

/-- Total number of coins: `Σ counts[i]`. -/
def total_coins (counts : List Nat) : Nat :=
  counts.foldl (fun acc c => acc + c) 0

/-- A coin-count vector is valid iff it covers all denominations, each count
    is non-negative, and the total value equals the target amount. -/
def is_valid_change (counts : List Nat) (coins : List Nat) (amount : Nat) : Prop :=
  counts.length = coins.length ∧
  total_amount counts coins = amount

/-- Correctness of coin_change: when the function succeeds,
    (1) there exists a valid change using exactly `result` coins;
    (2) every valid change uses at least `result` coins. -/
theorem coin_change_correct
    (coins : Slice Std.Usize) (amount : Std.Usize) (result : Std.Usize)
    (h : coin_change coins amount = ok result) :
    (∃ (counts : List Nat),
       is_valid_change counts (toNatList coins) amount.val ∧
       total_coins counts = result.val) ∧
    (∀ (counts : List Nat),
       is_valid_change counts (toNatList coins) amount.val →
       total_coins counts ≥ result.val) :=
by
  sorry