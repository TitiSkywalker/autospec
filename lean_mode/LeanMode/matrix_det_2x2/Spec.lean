import LeanMode.matrix_det_2x2.MatrixDet2x2
import Mathlib.LinearAlgebra.Matrix.Determinant.Basic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_det_2x2

set_option linter.unusedVariables false

/-!
# Specification: 2×2 Matrix Determinant

`det_2x2` computes the determinant of a 2×2 matrix stored in row-major
order.  The result is proved equal to Mathlib's `Matrix.det`.
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

/-- Correctness: `det_2x2 matrix` returns the Mathlib determinant of the
    2×2 matrix formed by `toMatrix`. -/
theorem det_2x2_correct (matrix : Slice Rat) (d : Rat)
    (h_len : matrix.length = 4)
    (h : det_2x2 matrix = ok d) :
    d = det (toMatrix matrix 2 2 h_len) := by
  sorry
