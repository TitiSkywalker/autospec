import LeanMode.integer_div_mod.IntegerDivMod
import Mathlib

open Aeneas Aeneas.Std Result
open integer_div_mod

set_option linter.unusedVariables false

/-!
# Specification: Floored division and modulo

`div_floor a b` and `mod_floor a b` compute the unique pair `(q, r)` with
`a = q * b + r` and `r` having the same sign as `b`: `0 ≤ r < b` when `b > 0`,
`b < r ≤ 0` when `b < 0`.
-/

/-- Floored division and modulo satisfy `a = q * b + r` with `r` of the same
    sign as `b`. -/
theorem div_mod_floor_correct (a b q r : Int)
    (hq : div_floor a b = ok q) (hr : mod_floor a b = ok r) :
    a = q * b + r ∧
    ((0 < b → 0 ≤ r ∧ r < b) ∧ (b < 0 → b < r ∧ r ≤ 0)) := by
  sorry
