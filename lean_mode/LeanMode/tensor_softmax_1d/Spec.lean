import LeanMode.tensor_softmax_1d.TensorSoftmax1d
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_softmax_1d

set_option linter.unusedVariables false

/-!
# Specification: 1D Softmax

`softmax input` computes `y[i] = exp(input[i]) / Σⱼ exp(input[j])`.
-/

/-- View a `Slice ℝ` of known length as `Fin len → ℝ`. -/
def toSliceVec (s : Slice ℝ) (len : Nat) (h : s.length = len) : Fin len → ℝ :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound

/-- View a `Vec ℝ` of known length as `Fin len → ℝ`. -/
def toVecVec (v : alloc.vec.Vec ℝ) (len : Nat) (h : v.length = len) : Fin len → ℝ :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    v[i.val]'h_bound


/-- Output length equals input length (for non-empty input). -/
theorem softmax_length
    (input : Slice ℝ) (result : alloc.vec.Vec ℝ)
    (h_softmax : softmax input = ok result)
    (hn : Slice.len input > 0#usize) :
    result.length = Slice.len input := by
  sorry


/-- `softmax` satisfies the mathematical definition:
    `out[i] = exp(input[i]) / Σⱼ exp(input[j])`. -/
theorem softmax_eq
    (input : Slice ℝ) (result : alloc.vec.Vec ℝ)
    (h_softmax : softmax input = ok result)
    (h_len_in : input.length = (Slice.len input).val)
    (h_len_out : result.length = (Slice.len input).val) :
    let n := (Slice.len input).val
    let inp := toSliceVec input n h_len_in
    let s : ℝ := ∑ j : Fin n, Real.exp (inp j)
    let out := toVecVec result n h_len_out
    ∀ (i : Fin n), out i = Real.exp (inp i) / s :=
by
  sorry
