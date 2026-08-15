import LeanMode.integer_perfect_square.IntegerPerfectSquare
import Mathlib

open Aeneas Aeneas.Std Result
open integer_perfect_square

set_option linter.unusedVariables false

/-!
# Specification: Perfect square test

`is_perfect_square n` is `true` iff `n` is a perfect square (∃k, k² = n).
-/

theorem is_perfect_square_correct (n : Nat) (result : Bool)
    (h : is_perfect_square n = ok result) :
    result = true ↔ ∃ k : Nat, k ^ 2 = n := by
  sorry
