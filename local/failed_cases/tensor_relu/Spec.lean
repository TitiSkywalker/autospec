import LeanMode.tensor_relu.TensorRelu
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_relu

set_option linter.unusedVariables false

/-!
# Specification: ReLU (Rectified Linear Unit)

`relu array` applies ReLU element-wise: `y[i] = max(0, x[i])`.

-/

/-- View a `Vec Rat` of known length as `Fin len → Rat`. -/
def toVec (v : alloc.vec.Vec Rat) (len : Nat) (h : v.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    v[i.val]'h_bound

/-- ReLU correctness: for each element i,
    `y[i] * (y[i] - x[i]) = 0` (either y[i]=0 or y[i]=x[i]) and `y[i] ≥ 0` (non-negative). -/
theorem relu_correct
    (input : alloc.vec.Vec Rat) (result : alloc.vec.Vec Rat)
    (h_relu : relu input = ok result)
    (h_len : result.length = input.length) :
    let n := input.length
    let x := toVec input n rfl
    let y := toVec result n h_len
    ∀ (i : Fin n), y i * (y i - x i) = 0 ∧ y i ≥ 0 :=
by
  sorry
