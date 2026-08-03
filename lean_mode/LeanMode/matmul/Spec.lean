import LeanMode.matmul.Matmul
import Mathlib.Data.Matrix.Basic
import Mathlib.Data.Fin.Basic

open Aeneas Aeneas.Std Result
open Matrix

set_option linter.unusedVariables false

/-- Convert a column-major flat slice to a Mathlib `Matrix`.
    `(i, j) ↦ s[i + j * rows]` -/
def toMatrix (s : Slice Rat) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) Rat :=
  λ i j =>
    have h_bound : i.val + j.val * rows < s.length := by
      rw [h]
      have hi : i.val < rows := i.is_lt
      have hj : j.val < cols := j.is_lt
      calc
        i.val + j.val * rows < rows + j.val * rows :=
          Nat.add_lt_add_right hi (j.val * rows)
        _ = (j.val + 1) * rows := by ring
        _ ≤ cols * rows :=
          Nat.mul_le_mul_right rows (Nat.succ_le_of_lt hj)
        _ = rows * cols := Nat.mul_comm cols rows
    s[i.val + j.val * rows]'h_bound

/-- `matmul` returns a `Vec` of length `m * n`. -/
theorem matmul_length (a b : Slice Rat) (m k n : Usize) (result : alloc.vec.Vec Rat)
    (h : matmul.matmul a b m k n = ok result) :
    result.length = m.val * n.val :=
by
  sorry

/-- Correctness of column-major matrix multiplication: `C = A * B`. -/
theorem matmul_correct
    (a b : Slice Rat)
    (m k n : Usize)
    (h_len_a : a.length = m.val * k.val)
    (h_len_b : b.length = k.val * n.val)
    (h_result : matmul.matmul a b m k n = ok result) :
    toMatrix (alloc.vec.Vec.deref result) m.val n.val
      (matmul_length a b m k n result h_result) =
    toMatrix a m.val k.val h_len_a * toMatrix b k.val n.val h_len_b :=
by
  sorry
