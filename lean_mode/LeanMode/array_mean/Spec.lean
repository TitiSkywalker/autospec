import LeanMode.array_mean.ArrayMean
import Mathlib

open Aeneas Aeneas.Std Result
open array_mean

set_option linter.unusedVariables false

/-!
# Specification: Vector Mean

`mean input` returns the arithmetic mean of all elements in the slice.
-/

/-- View a `Slice Rat` of known length as `Fin len → Rat`. -/
def toVec (s : Slice Rat) (len : Nat) (h : s.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound


/-- Correctness: `mean input = ok ((∑_{j < n} input[j]) / n)`. -/
theorem mean_eq
    (input : Slice Rat) (result : Rat)
    (h_mean : mean input = ok result)
    (h_len : input.length = (Slice.len input).val) :
    let n := (Slice.len input).val
    result = (∑ j : Fin n, (toVec input n h_len) j) / (n : Rat) := by
  sorry
