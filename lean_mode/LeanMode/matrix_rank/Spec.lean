import LeanMode.matrix_rank.MatrixRank
import Mathlib.LinearAlgebra.Matrix.Rank
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_rank

set_option linter.unusedVariables false

/-!
# Specification: Matrix Rank

`rank` computes the rank of an n×m matrix stored in row-major order via
Gaussian elimination with partial pivoting.
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

/-- Correctness: the rank of the matrix equals Mathlib's `Matrix.rank`. -/
theorem rank_correct (matrix : Slice Rat) (nrows ncols : Std.Usize) (r : Std.Usize)
    (h_len : matrix.length = nrows.val * ncols.val)
    (h : rank matrix nrows ncols = ok r) :
    r.val = Matrix.rank (toMatrix matrix nrows.val ncols.val h_len) := by
  sorry
