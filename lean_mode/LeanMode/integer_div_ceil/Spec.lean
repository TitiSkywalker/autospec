import LeanMode.integer_div_ceil.IntegerDivCeil
import Mathlib

open Aeneas Aeneas.Std Result
open integer_div_ceil

set_option linter.unusedVariables false

/-!
# Specification: Ceiling Division (unsigned)

`div_ceil_u64 lhs rhs` computes `⌈lhs/rhs⌉` for `rhs > 0`.
-/

/-- Correctness of ceiling division: `result = ⌈lhs/rhs⌉ = (lhs+rhs-1)/rhs`. -/
theorem div_ceil_u64_correct (lhs rhs result : Int)
    (h : div_ceil_u64 lhs rhs = ok result) :
    result = (lhs + rhs - 1) / rhs := by
  sorry
