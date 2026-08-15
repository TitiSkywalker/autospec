import LeanMode.integer_div_mod.IntegerDivMod
import Mathlib

open Aeneas Aeneas.Std Result
open integer_div_mod

set_option linter.unusedVariables false

/-!
# Specification: Euclidean (floored) division/modulo and ceiling division

`div_floor a b` and `mod_floor a b` compute the unique pair `(q, r)` with
`a = q * b + r` and `0 ≤ r < |b|`.  `div_ceil a b` computes `⌈a/b⌉`.
-/

/-- Floored division and modulo satisfy `a = q * b + r` with `0 ≤ r < |b|`. -/
theorem div_mod_floor_correct (a b q r : Int)
    (hq : div_floor a b = ok q) (hr : mod_floor a b = ok r) :
    a = q * b + r ∧ 0 ≤ r ∧ r < (Int.natAbs b : Int) := by
  sorry

/-- Ceiling division satisfies `⌈a/b⌉ = -⌊-a/b⌋`. -/
theorem div_ceil_correct (a b q : Int)
    (hq : div_ceil a b = ok q) :
    q = -(Int.fdiv (-a) b) := by
  sorry
