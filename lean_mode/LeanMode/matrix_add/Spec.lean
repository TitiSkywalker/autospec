import LeanMode.matrix_add.MatrixAdd
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_add

set_option linter.unusedVariables false

/-!
# Specification: Matrix Addition

`matadd a b m n` = A + B (element-wise, row-major).
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

/-- `matadd` returns a vec of length `m * n`. -/
theorem matadd_length (a b : Slice Rat) (m n : Usize) (result : alloc.vec.Vec Rat)
    (h : matadd a b m n = ok result) :
    result.length = m.val * n.val := by
  sorry

/-- Correctness: `C = A + B` element-wise. -/
theorem matadd_correct
    (a b : Slice Rat)
    (m n : Usize)
    (h_len_a : a.length = m.val * n.val)
    (h_len_b : b.length = m.val * n.val)
    (result : alloc.vec.Vec Rat)
    (h_result : matadd a b m n = ok result) :
    toMatrix (alloc.vec.Vec.deref result) m.val n.val (matadd_length a b m n result h_result) =
    toMatrix a m.val n.val h_len_a + toMatrix b m.val n.val h_len_b := by
  sorry
