import LeanMode.integer_mod_pow.IntegerModPow
import Mathlib

open Aeneas Aeneas.Std Result
open integer_mod_pow

set_option linter.unusedVariables false

/-!
# Specification: Modular Exponentiation

`mod_pow base exp modulus` computes `base^exp mod modulus` by
square-and-multiply (requires `modulus > 0`).
-/

/-- Correctness of modular exponentiation (for modulus ≥ 2). -/
theorem mod_pow_correct (base exp modulus result : Nat)
    (h_mod : 1 < modulus)
    (h : mod_pow base exp modulus = ok result) :
    result = (base ^ exp) % modulus := by
  sorry
