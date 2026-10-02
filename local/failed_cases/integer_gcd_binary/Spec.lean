import LeanMode.integer_gcd_binary.IntegerGcdBinary
import Mathlib

open Aeneas Aeneas.Std Result
open integer_gcd_binary

set_option linter.unusedVariables false

/-!
# Specification: Stein's binary GCD

`gcd_binary a b` computes the greatest common divisor of `a` and `b`.
-/

theorem gcd_binary_correct (a b result : Nat)
    (h : gcd_binary a b = ok result) :
    result = Nat.gcd a b := by
  sorry
