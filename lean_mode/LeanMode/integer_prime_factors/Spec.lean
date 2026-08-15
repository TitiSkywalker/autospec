import LeanMode.integer_prime_factors.IntegerPrimeFactors
import Mathlib

open Aeneas Aeneas.Std Result
open integer_prime_factors

set_option linter.unusedVariables false

/-!
# Specification: Prime factorization

`prime_factors n` returns the prime factors of `n` in non-decreasing order
(with repetition), so their product equals `n`.
-/

theorem prime_factors_correct (n : Nat) (result : alloc.vec.Vec Nat)
    (h : prime_factors n = ok result) :
    (alloc.vec.Vec.deref result : List Nat).prod = n ∧
    (∀ p : Nat, p ∈ (alloc.vec.Vec.deref result : List Nat) → Nat.Prime p) ∧
    List.Pairwise (· ≤ ·) (alloc.vec.Vec.deref result : List Nat) := by
  sorry
