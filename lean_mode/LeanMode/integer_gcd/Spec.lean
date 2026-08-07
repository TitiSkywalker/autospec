import LeanMode.integer_gcd.IntegerGcd
import Mathlib

open Aeneas Aeneas.Std Result
open integer_gcd

set_option linter.unusedVariables false

/-!
# Specification: Integer GCD

`compute_gcd` computes the greatest common divisor of two integers
using the Euclidean algorithm.  The result is always non-negative
and matches `Int.gcd` from Mathlib.
-/

/-- Correctness of GCD: the result matches `Int.gcd`. -/
theorem compute_gcd_correct (a b result : Int)
    (h : compute_gcd a b = ok result) :
    result = Int.gcd a b := by
  sorry
