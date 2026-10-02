import LeanMode.integer_linear_sieve.IntegerLinearSieve
import Mathlib

open Aeneas Aeneas.Std Result
open integer_linear_sieve

set_option linter.unusedVariables false

/-!
# Specification: Linear sieve

`linear_sieve n` returns `(primes, mpf)` where `primes` lists the primes `≤ n`
and `mpf[k]` is the least prime factor of `k` (for `k ≥ 2`).
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (·.val)

theorem linear_sieve_correct (n : Std.Usize)
    (primes mpf : alloc.vec.Vec Std.Usize)
    (h : linear_sieve n = ok (primes, mpf)) :
    toNatList (alloc.vec.Vec.deref primes) =
        (List.range (n.val + 1)).filter (fun k => Nat.Prime k) ∧
    (∀ k : Nat, 2 ≤ k → k ≤ n.val →
      (toNatList (alloc.vec.Vec.deref mpf))[k]? = some (Nat.minFac k)) := by
  sorry
