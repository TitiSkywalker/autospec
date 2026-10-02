import LeanMode.integer_euler_totient_sieve.IntegerEulerTotientSieve
import Mathlib

open Aeneas Aeneas.Std Result
open integer_euler_totient_sieve

set_option linter.unusedVariables false

/-!
# Specification: Batch Euler totient

`euler_totient_sieve n` returns `[φ(1), φ(2), ..., φ(n)]`.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (·.val)

theorem euler_totient_sieve_correct (n : Std.Usize) (result : alloc.vec.Vec Std.Usize)
    (h : euler_totient_sieve n = ok result) :
    toNatList (alloc.vec.Vec.deref result) =
      List.map Nat.totient (List.drop 1 (List.range (n.val + 1))) := by
  sorry
