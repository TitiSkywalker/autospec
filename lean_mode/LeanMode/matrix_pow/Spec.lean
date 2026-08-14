import LeanMode.matrix_pow.MatrixPow
import Mathlib

open Aeneas Aeneas.Std Result
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Matrix power

`matrix_pow matrix n exp` computes `matrix^exp` for an n×n matrix via
exponentiation by squaring.  `matrix^0` is the identity.
-/

/-- Row-major flat `Slice Rat` → `Matrix (Fin rows) (Fin cols) Rat`.
    Index: `element (row, col)` is at `row * cols + col`. -/
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

/-- `matrix_pow` returns an n×n result. -/
theorem matrix_pow_length
    (matrix : Slice Rat) (n exp : Std.Usize) (result : alloc.vec.Vec Rat)
    (h : matrix_pow.matrix_pow matrix n exp = ok result) :
    result.length = n.val * n.val :=
by
  sorry

/-- Correctness of matrix power: the result is `A ^ exp`. -/
theorem matrix_pow_correct
    (matrix : Slice Rat) (n exp : Std.Usize) (result : alloc.vec.Vec Rat)
    (h : matrix_pow.matrix_pow matrix n exp = ok result)
    (h_len : matrix.length = n.val * n.val) :
    toMatrix (alloc.vec.Vec.deref result) n.val n.val
      (matrix_pow_length matrix n exp result h) =
    (toMatrix matrix n.val n.val h_len) ^ exp.val :=
by
  sorry
