import LeanMode.integer_gcd.IntegerGcd
import Mathlib

open Aeneas Aeneas.Std Result
open integer_gcd

set_option linter.unusedVariables false

/-!
# Specification: Integer GCD

`compute_gcd` computes the greatest common divisor of two natural numbers
using the Euclidean algorithm.  The result matches `Nat.gcd` from Mathlib.
-/

/-- Correctness of GCD: the result matches `Nat.gcd`. -/
theorem compute_gcd_correct (a b result : Nat)
    (h : compute_gcd a b = ok result) :
    result = Nat.gcd a b := by
  sorry
