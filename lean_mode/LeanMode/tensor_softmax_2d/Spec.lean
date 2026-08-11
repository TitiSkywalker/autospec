import LeanMode.tensor_softmax_2d.TensorSoftmax2d
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_softmax_2d

set_option linter.unusedVariables false

/-!
# Specification: 2D Row-wise Softmax

`softmax2d input rows cols` computes row-wise softmax on an H×W matrix
stored in row-major order.

For each row r and column j:
  out[r][j] = exp(input[r][j]) / Σₖ exp(input[r][k])
-/

/-- View a flat row-major `Slice ℝ` as a matrix: `Fin rows → Fin cols → ℝ`. -/
def toMatrix (s : Slice ℝ) (rows cols : Nat) (h : s.length = rows * cols) :
    Fin rows → Fin cols → ℝ :=
  λ i j =>
    have h_bound : i.val * cols + j.val < s.length := by
      rw [h]
      have hi : i.val < rows := i.is_lt
      have hj : j.val < cols := j.is_lt
      calc
        i.val * cols + j.val < i.val * cols + cols := Nat.add_lt_add_left hj _
        _ = (i.val + 1) * cols := by ring
        _ ≤ rows * cols := Nat.mul_le_mul_right _ (Nat.succ_le_of_lt hi)
    s[i.val * cols + j.val]'h_bound

/-- Output length equals input length (for non-empty input). -/
theorem softmax2d_length
    (input : Slice ℝ) (rows cols : Usize) (result : alloc.vec.Vec ℝ)
    (h_softmax : softmax2d input rows cols = ok result)
    (hr : rows > 0#usize) (hc : cols > 0#usize) :
    result.length = Slice.len input := by
  sorry

/-- `softmax2d` satisfies row-wise softmax:
    `out[r][j] = exp(input[r][j]) / Σₖ exp(input[r][k])`. -/
theorem softmax2d_eq
    (input : Slice ℝ) (rows cols : Usize) (result : alloc.vec.Vec ℝ)
    (h_softmax : softmax2d input rows cols = ok result)
    (h_len_in : input.length = rows.val * cols.val)
    (h_len_out : result.length = rows.val * cols.val) :
    let H := rows.val; let W := cols.val
    let inp := toMatrix input H W h_len_in
    let out := toMatrix (alloc.vec.Vec.deref result) H W h_len_out
    ∀ (r : Fin H) (j : Fin W),
      out r j = Real.exp (inp r j) / (∑ k : Fin W, Real.exp (inp r k)) :=
by
  sorry
