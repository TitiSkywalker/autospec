import LeanMode.matrix_kronecker.MatrixKronecker
import Mathlib.Data.Matrix.Basic
import Mathlib.Data.Fin.Basic
import Mathlib.LinearAlgebra.Matrix.Kronecker
import Mathlib.Logic.Equiv.Fin.Basic

open Aeneas Aeneas.Std Result
open Matrix
open scoped Kronecker

set_option linter.unusedVariables false

/-!
# Specification: Kronecker product of matrices

`K = A ⊗ₖ B`  where `⊗ₖ` is the Mathlib Kronecker product.
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

/-- `kron` returns a `Vec` of the expected length. -/
theorem kron_length (a b : Slice Rat) (a_rows a_cols b_rows b_cols : Usize)
    (result : alloc.vec.Vec Rat)
    (h : matrix_kronecker.kron a a_rows a_cols b b_rows b_cols = ok result) :
    result.length = (a_rows.val * b_rows.val) * (a_cols.val * b_cols.val) :=
  by
    sorry

-- Separate section for the correctness theorem and its validation

/-- Correctness of Kronecker product: `K = A ⊗ₖ B`.

The Rust `kron` uses row-major flat indexing.  `finProdFinEquiv.symm`
maps the flat index back to a product pair `(row_of_A, row_of_B)` (and
similarly for columns), aligning the output with Mathlib's
`Matrix.kronecker` which uses product indices. -/
theorem kron_correct
    (a b : Slice Rat)
    (a_rows a_cols b_rows b_cols : Usize)
    (h_len_a : a.length = a_rows.val * a_cols.val)
    (h_len_b : b.length = b_rows.val * b_cols.val)
    (result : alloc.vec.Vec Rat)
    (h_result : matrix_kronecker.kron a a_rows a_cols b b_rows b_cols = ok result) :
    (toMatrix (alloc.vec.Vec.deref result)
        (a_rows.val * b_rows.val) (a_cols.val * b_cols.val)
        (kron_length a b a_rows a_cols b_rows b_cols result h_result)).reindex
      (finProdFinEquiv (m := a_rows.val) (n := b_rows.val)).symm
      (finProdFinEquiv (m := a_cols.val) (n := b_cols.val)).symm =
    Matrix.kronecker (toMatrix a a_rows.val a_cols.val h_len_a)
                     (toMatrix b b_rows.val b_cols.val h_len_b) :=
  by
    sorry
