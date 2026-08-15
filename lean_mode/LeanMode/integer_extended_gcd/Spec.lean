import LeanMode.integer_extended_gcd.IntegerExtendedGcd
import Mathlib

open Aeneas Aeneas.Std Result
open integer_extended_gcd

set_option linter.unusedVariables false

/-!
# Specification: Extended Euclidean Algorithm (Bézout's identity)

`extended_gcd a b` returns `(g, x, y)` such that `g = gcd a b` and the
Bézout identity `a * x + b * y = g` holds.
-/

/-- Correctness of the extended Euclidean algorithm: the first component is
    the gcd and the coefficients satisfy Bézout's identity. -/
theorem extended_gcd_correct (a b g x y : Int)
    (h : extended_gcd a b = ok (g, x, y)) :
    g = Int.gcd a b ∧ a * x + b * y = g := by
  sorry
