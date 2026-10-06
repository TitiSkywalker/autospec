import LeanMode.array_sum.ArraySum
import Mathlib

open Aeneas Aeneas.Std Result
open array_sum

set_option linter.unusedVariables false

/-!
# Specification: Vector Sum

`sum input` returns the total of all elements in the slice.
-/

/-- View a `Slice Rat` of known length as `Fin len → Rat`. -/
def toVec (s : Slice Rat) (len : Nat) (h : s.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound


/-- Correctness: `sum input = ok (∑_{j < len(input)} input[j])`. -/
theorem sum_eq
    (input : Slice Rat) (result : Rat)
    (h_sum : sum input = ok result)
    (h_len : input.length = (Slice.len input).val) :
    let n := (Slice.len input).val
    result = (∑ j : Fin n, (toVec input n h_len) j) := by
  sorry
