import LeanMode.integer_pow.IntegerPow
import Mathlib

open Aeneas Aeneas.Std Result
open integer_pow

set_option linter.unusedVariables false

/-!
# Specification: Binary Exponentiation

`pow a e` computes `a^e` by exponentiation by squaring.
-/

/-- Correctness of binary exponentiation. -/
theorem pow_correct (a e result : Nat)
    (h : pow a e = ok result) :
    result = a ^ e := by
  sorry
