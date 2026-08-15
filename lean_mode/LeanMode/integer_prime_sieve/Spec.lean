import LeanMode.integer_prime_sieve.IntegerPrimeSieve
import Mathlib

open Aeneas Aeneas.Std Result
open integer_prime_sieve

set_option linter.unusedVariables false

/-!
# Specification: Sieve of Eratosthenes

`prime_sieve n` returns all primes `≤ n` in increasing order.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (·.val)

theorem prime_sieve_correct (n : Std.Usize) (result : alloc.vec.Vec Std.Usize)
    (h : prime_sieve n = ok result) :
    List.Pairwise (· < ·) (toNatList (alloc.vec.Vec.deref result)) ∧
    (∀ p : Nat, p ∈ toNatList (alloc.vec.Vec.deref result) ↔ (p ≤ n.val ∧ Nat.Prime p)) := by
  sorry
