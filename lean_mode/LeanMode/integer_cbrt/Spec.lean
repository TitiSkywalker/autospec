import LeanMode.integer_cbrt.IntegerCbrt
import Mathlib

open Aeneas Aeneas.Std Result
open integer_cbrt

set_option linter.unusedVariables false

/-!
# Specification: Integer cube root

`icbrt x` computes `⌊∛x⌋`, the largest `r` with `r³ ≤ x < (r+1)³`.
-/

theorem icbrt_correct (x result : Nat)
    (h : icbrt x = ok result) :
    result ^ 3 ≤ x ∧ x < (result + 1) ^ 3 := by
  sorry
