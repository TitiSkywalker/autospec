import LeanMode.integer_lcm.IntegerLcm
import Mathlib

open Aeneas Aeneas.Std Result
open integer_lcm

set_option linter.unusedVariables false

/-!
# Specification: Least Common Multiple

`lcm a b` computes the least common multiple, satisfying
`lcm a b * gcd a b = a * b`.
-/

/-- Correctness of the least common multiple: `lcm * gcd = a * b`. -/
theorem lcm_correct (a b result : Nat)
    (h : integer_lcm.lcm a b = ok result) :
    result * (Nat.gcd a b) = a * b := by
  sorry
