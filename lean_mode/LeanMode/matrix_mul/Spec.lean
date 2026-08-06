import LeanMode.matrix_mul.MatrixMul
import Mathlib.Data.Matrix.Basic
import Mathlib.Data.Fin.Basic

open Aeneas Aeneas.Std Result
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Matrix multiplication

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

/-- `matmul` returns a `Vec` of length `m * n`. -/
theorem matmul_length (a b : Slice Rat) (m k n : Usize) (result : alloc.vec.Vec Rat)
    (h : matrix_mul.matmul a b m k n = ok result) :
    result.length = m.val * n.val :=
by
  sorry

/-- Correctness of row-major matrix multiplication:  `C = A * B`. -/
theorem matmul_correct
    (a b : Slice Rat)
    (m k n : Usize)
    (h_len_a : a.length = m.val * k.val)
    (h_len_b : b.length = k.val * n.val)
    (result : alloc.vec.Vec Rat)
    (h_result : matrix_mul.matmul a b m k n = ok result) :
    toMatrix (alloc.vec.Vec.deref result) m.val n.val
      (matmul_length a b m k n result h_result) =
    toMatrix a m.val k.val h_len_a * toMatrix b k.val n.val h_len_b :=
by
  sorry
