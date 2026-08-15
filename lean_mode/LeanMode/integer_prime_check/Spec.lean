import LeanMode.integer_prime_check.IntegerPrimeCheck
import Mathlib

open Aeneas Aeneas.Std Result
open integer_prime_check

set_option linter.unusedVariables false

/-!
# Specification: Primality test

`is_prime n` is `true` iff `n` is prime.
-/

theorem is_prime_correct (n : Nat) (result : Bool)
    (h : is_prime n = ok result) :
    result = true ↔ Nat.Prime n := by
  sorry
