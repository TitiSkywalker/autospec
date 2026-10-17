import LeanMode.integer_factorial.IntegerFactorial
import Mathlib

open Aeneas Aeneas.Std Result
open integer_factorial

set_option linter.unusedVariables false

/-!
# Specification: Factorial

`factorial n` computes `n!`.
-/

/-- Correctness of the iterative factorial. -/
theorem factorial_correct (n result : Nat)
    (h : factorial n = ok result) :
    result = Nat.factorial n := by
  sorry
