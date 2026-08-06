import LeanMode.gcd.Gcd
import Mathlib

open Aeneas Aeneas.Std Result
open gcd

set_option linter.unusedVariables false

/-!
# Specification: GCD

The result of `compute_gcd` equals `Nat.gcd` from Mathlib.
-/

/-- Correctness of GCD: the result matches `Nat.gcd`. -/
theorem compute_gcd_correct (a b result : U64)
    (h : compute_gcd a b = ok result) :
    result.val = Nat.gcd a.val b.val := by
  sorry
