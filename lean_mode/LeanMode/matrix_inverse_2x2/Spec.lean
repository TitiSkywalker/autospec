import LeanMode.matrix_inverse_2x2.MatrixInverse2x2
import Mathlib.Data.Matrix.Basic
import Mathlib.Data.Fin.Basic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_inverse_2x2

set_option linter.unusedVariables false

/-!
# Specification: 2×2 Matrix Inverse

`invert_2x2` returns `fail .panic` when the determinant is zero (singular
matrix) and `ok result` otherwise.

Hence `h : invert_2x2 matrix = ok result` already encodes the invertibility
precondition — **no separate `det ≠ 0` hypothesis is required.**
-/

/-- Row-major flat slice → Mathlib `Matrix`.  `(i, j) ↦ s[i * cols + j]` -/
def toMatrix (s : Slice Rat) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) Rat :=
  λ i j =>
    have h_bound : i.val * cols + j.val < s.length := by
      rw [h]
      have hi : i.val < rows := i.is_lt
      have hj : j.val < cols := j.is_lt
      calc
        i.val * cols + j.val < i.val * cols + cols :=
          Nat.add_lt_add_left hj (i.val * cols)
        _ = (i.val + 1) * cols := by ring
        _ ≤ rows * cols :=
          Nat.mul_le_mul_right cols (Nat.succ_le_of_lt hi)
    s[i.val * cols + j.val]'h_bound

/-- `ok result` implies the output has length 4. -/
theorem invert_2x2_length (matrix : Slice Rat) (result : alloc.vec.Vec Rat)
    (h : invert_2x2 matrix = ok result) :
    result.length = 4 := by
  sorry

/-- Correctness of 2×2 matrix inverse.

    When `invert_2x2 matrix` returns `ok result`, the result is the
    matrix inverse: `M * R = I`. -/
theorem invert_2x2_correct (matrix : Slice Rat) (result : alloc.vec.Vec Rat)
    (h_len : matrix.length = 4)
    (h_res_len : (alloc.vec.Vec.deref result).length = 4)
    (h : invert_2x2 matrix = ok result) :
    toMatrix matrix 2 2 h_len * toMatrix (alloc.vec.Vec.deref result) 2 2 h_res_len
      = (1 : Matrix (Fin 2) (Fin 2) Rat) := by
  sorry
