import LeanMode.matrix_inverse.MatrixInverse
import Mathlib.LinearAlgebra.Matrix.Determinant.Basic
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_inverse

set_option linter.unusedVariables false

/-!
# Specification: n×n Matrix Inverse

`matrix_inverse` computes the inverse of an n×n matrix via Gauss-Jordan
elimination with partial pivoting.

The function asserts (`massert`) if the matrix is singular, so the Lean
translation returns `Result (Vec Rat)`: `ok inv` on success, `fail` on
singular input.  Consequently `ok` **proves** the matrix was invertible.
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

/-- `A⁻¹` is the **inverse** of `A`: both `A * A⁻¹ = I` and `A⁻¹ * A = I`. -/
def IsInverse {n : Nat} (a a_inv : Matrix (Fin n) (Fin n) Rat) : Prop :=
  a * a_inv = 1 ∧ a_inv * a = 1

/-! ## Correctness theorem (placeholder) -/

/-- **`matrix_inverse_correct`**:
    If `matrix` is an n×n square matrix and `matrix_inverse matrix n`
    returns `ok inv_vec`, then `toMatrix inv_vec` is the inverse of
    `toMatrix matrix` — i.e. `A * A⁻¹ = I` and `A⁻¹ * A = I`.

    The `ok` result itself proves the original matrix was invertible
    (the algorithm would have returned `fail` otherwise).

    Proof requires formalizing the Gauss-Jordan elimination algorithm:
    - Row operations preserve the solution of `A * X = I`
    - After elimination, the augmented matrix is `[I | A⁻¹]`
    - `massert (max_val ≠ 0)` → pivot is non-zero → matrix is invertible -/
theorem matrix_inverse_correct (matrix : Slice Rat) (n : Std.Usize)
    (inv_vec : alloc.vec.Vec Rat)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h : matrix_inverse matrix n = ok inv_vec)
    (h_inv_len : (alloc.vec.Vec.deref inv_vec).val.length = (n.val : Nat) * (n.val : Nat)) :
    IsInverse (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len)
              (toMatrix (alloc.vec.Vec.deref inv_vec) (n.val : Nat) (n.val : Nat) h_inv_len) := by
  sorry
