import LeanMode.array_var.VectorVar
import Mathlib

open Aeneas Aeneas.Std Result
open array_var

set_option linter.unusedVariables false

/-!
# Specification: Vector Variance

`var input` returns the population variance of all elements in the slice.
-/

/-- View a `Slice Rat` of known length as `Fin len → Rat`. -/
def toVec (s : Slice Rat) (len : Nat) (h : s.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound


/-- Correctness: `var input = ok ((∑_{j < n} (input[j] - μ)²) / n)` where `μ` is the mean. -/
theorem var_eq
    (input : Slice Rat) (result : Rat)
    (h_var : var input = ok result)
    (h_len : input.length = (Slice.len input).val) :
    let n := (Slice.len input).val
    let μ := (∑ j : Fin n, (toVec input n h_len) j) / (n : Rat)
    result = (∑ j : Fin n, ((toVec input n h_len) j - μ) * ((toVec input n h_len) j - μ)) / (n : Rat) := by
  sorry
