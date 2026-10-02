import LeanMode.integer_sqrt.IntegerSqrt
import Mathlib

open Aeneas Aeneas.Std Result
open integer_sqrt

set_option linter.unusedVariables false

/-!
# Specification: Integer Square Root

`isqrt x` computes `⌊√x⌋`, the largest `r` with `r² ≤ x < (r+1)²`.
-/

/-- Correctness of the integer square root. -/
theorem isqrt_correct (x result : Nat)
    (h : isqrt x = ok result) :
    result ^ 2 ≤ x ∧ x < (result + 1) ^ 2 := by
  sorry
