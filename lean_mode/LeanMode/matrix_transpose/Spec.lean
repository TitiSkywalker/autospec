import LeanMode.matrix_transpose.MatrixTranspose
import Mathlib.LinearAlgebra.Matrix.Determinant.Basic
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_transpose

set_option linter.unusedVariables false

/-!
# Specification: n×n Matrix Transpose

`transpose` computes the transpose of an n×n matrix:
`A^T[i][j] = A[j][i]`.
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

/-! ## Correctness theorem (placeholder) -/

/-- **`transpose_correct`**:
    If `matrix` is an n×n square matrix, `transpose matrix n` returns
    `ok t_vec`, and `t_vec` has length `n·n`, then `toMatrix t_vec`
    equals the Mathlib transpose of `toMatrix matrix`. -/
theorem transpose_correct (matrix : Slice Rat) (n : Std.Usize)
    (t_vec : alloc.vec.Vec Rat)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h : transpose matrix n = ok t_vec)
    (h_t_len : (alloc.vec.Vec.deref t_vec).val.length = (n.val : Nat) * (n.val : Nat)) :
    (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len)ᵀ =
    toMatrix (alloc.vec.Vec.deref t_vec) (n.val : Nat) (n.val : Nat) h_t_len := by
  sorry
