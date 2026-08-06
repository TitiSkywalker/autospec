import LeanMode.matrix_det.MatrixDet
import Mathlib.LinearAlgebra.Matrix.Determinant.Basic
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_det

set_option linter.unusedVariables false

/-!
# Specification: n×n Matrix Determinant

`det` computes the determinant of an n×n matrix stored in row-major
order via Gaussian elimination with partial pivoting.
-/

/-- Row-major flat slice → Mathlib `Matrix`.
    `(i, j) ↦ s[i * cols + j]` -/
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

/-- Correctness: the result of `det matrix n` equals Mathlib's `Matrix.det`
    of the n×n matrix. -/
theorem det_correct (matrix : Slice Rat) (n : Std.Usize) (d : Rat)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h : det matrix n = ok d) :
    d = det (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len) := by
  sorry
