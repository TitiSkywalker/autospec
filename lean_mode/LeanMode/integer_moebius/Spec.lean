import LeanMode.integer_moebius.IntegerMoebius
import Mathlib

open Aeneas Aeneas.Std Result
open integer_moebius

set_option linter.unusedVariables false

/-!
# Specification: Möbius function μ(n)

`moebius n` computes the Möbius function: `0` if `n` has a square factor
(a prime `p` with `p² ∣ n`), otherwise `(-1)^k` where `k` is the number of
distinct prime factors of `n`.
-/

/-- `n` has a square factor: some prime `p` satisfies `p² ∣ n`. -/
def has_square_factor (n : Nat) : Prop :=
  ∃ p : Nat, Nat.Prime p ∧ p * p ∣ n

/-- The Möbius function is always one of `-1`, `0`, or `1`. -/
theorem moebius_range (n : Nat) (result : Int) (h : moebius n = ok result) :
    result = -1 ∨ result = 0 ∨ result = 1 := by
  sorry

/-- `moebius n = 0` exactly when `n` has a square factor (for `n > 0`). -/
theorem moebius_square_free (n : Nat) (result : Int) (hn : 0 < n)
    (h : moebius n = ok result) :
    (result = 0) ↔ has_square_factor n := by
  sorry
