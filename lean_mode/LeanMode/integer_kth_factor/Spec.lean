import LeanMode.integer_kth_factor.IntegerKthFactor
import Mathlib

open Aeneas Aeneas.Std Result
open integer_kth_factor

set_option linter.unusedVariables false

/-!
# Specification: k-th factor

`kth_factor n k` returns the `k`-th smallest positive divisor of `n`
(1-indexed), or `-1` if `n` has fewer than `k` divisors.
-/

/-- The positive divisors of `n` (as a finite set). -/
def divisors_finset (n : Nat) : Finset Nat :=
  (Finset.range (n + 1)).filter (fun d => d ∣ n)

theorem kth_factor_correct (n k result : Int) (hn : 0 < n) (hk : 0 < k)
    (h : kth_factor n k = ok result) :
    (result = -1 ∧ (divisors_finset n.natAbs).card < k.natAbs) ∨
    (0 < result ∧ result ∣ n ∧
      ((divisors_finset n.natAbs).filter (fun d => d < result.natAbs)).card = k.natAbs - 1) := by
  sorry
