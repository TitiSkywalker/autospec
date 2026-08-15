import LeanMode.integer_factors.IntegerFactors
import Mathlib

open Aeneas Aeneas.Std Result
open integer_factors

set_option linter.unusedVariables false

/-!
# Specification: All divisors

`divisors n` returns all positive divisors of `n` in increasing order.
-/

theorem divisors_correct (n : Nat) (result : alloc.vec.Vec Nat)
    (h : divisors n = ok result) :
    List.Pairwise (· < ·) (alloc.vec.Vec.deref result : List Nat) ∧
    (∀ d : Nat, d ∈ (alloc.vec.Vec.deref result : List Nat) ↔ d ∣ n) := by
  sorry
