import LeanMode.matrix_vector_product.MatrixVectorProduct
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_vector_product

set_option linter.unusedVariables false

/-!
# Specification: Matrix-Vector Multiplication

`matvec a x m n` = A * x (row-major, A: m×n, x: n, result: m).
-/

/-- Row-major flat slice → `Matrix (Fin rows) (Fin cols) Rat`. -/
def toMatrix (s : Slice Rat) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) Rat :=
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

/-- Flat slice → `Fin n → Rat` vector. -/
def toVec (s : Slice Rat) (len : Nat) (h : s.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound

/-- `matvec` returns a vec of length `m`. -/
theorem matvec_length (a x : Slice Rat) (m n : Usize) (result : alloc.vec.Vec Rat)
    (h : matvec a x m n = ok result) :
    (alloc.vec.Vec.deref result).length = m.val := by
  sorry

/-- Correctness: `y = A * x`. -/
theorem matvec_correct
    (a x : Slice Rat)
    (m n : Usize)
    (h_len_a : a.length = m.val * n.val)
    (h_len_x : x.length = n.val)
    (result : alloc.vec.Vec Rat)
    (h_result : matvec a x m n = ok result) :
    toVec (alloc.vec.Vec.deref result) m.val (matvec_length a x m n result h_result) =
    Matrix.mulVec (toMatrix a m.val n.val h_len_a) (toVec x n.val h_len_x) := by
  sorry
