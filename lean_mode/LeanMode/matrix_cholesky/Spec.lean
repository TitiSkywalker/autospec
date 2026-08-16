import LeanMode.matrix_cholesky.MatrixCholesky
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_cholesky
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Cholesky Decomposition

`cholesky_decomposition` factorises a real symmetric positive-definite n×n
matrix A (column-major) into `Cholesky { l }` where:

> **A = L · Lᵀ**

with `L` — n×n lower-triangular matrix with a positive diagonal.

The input must be **symmetric** (`IsSymmetric` hypothesis below); positive-
definiteness is *not* carried as a hypothesis — the Rust `assert!` rejects a
non-positive diagonal pivot, so an `ok` result already implies the matrix was
positive-definite.
-/

/-- Symmetry of a square matrix, expressed on the `toMatrix` view. -/
def IsSymmetric {n : Nat} (a : Matrix (Fin n) (Fin n) ℝ) : Prop :=
  ∀ i j, a i j = a j i

/-! ### helpers -/

/-- Column-major flat `Slice ℝ` → `Matrix (Fin rows) (Fin cols) ℝ`.
    Index: `element (row, col)` is at `row + col * nrows`. -/
def toMatrix (s : Slice ℝ) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) ℝ :=
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

/-! ### correctness -/

/-- **Cholesky output length.**

    The returned `l` has the correct length n×n. -/
theorem cholesky_length
    (matrix : Slice ℝ) (n : Std.Usize) (result : Cholesky)
    (h : cholesky_decomposition matrix n = ok result) :
    (alloc.vec.Vec.deref result.l).length = n.val * n.val :=
by
  sorry

/-- **Cholesky decomposition correctness.**

    If `matrix` is a symmetric n×n matrix (`hsym`) and
    `cholesky_decomposition matrix n` returns `ok result`, then the returned
    `l` (n×n), viewed as a matrix via `toMatrix`, is lower-triangular and
    satisfies `A = L · Lᵀ`. -/
theorem cholesky_correct
    (matrix : Slice ℝ) (n : Std.Usize) (result : Cholesky)
    (h : cholesky_decomposition matrix n = ok result)
    (h_len : matrix.length = n.val * n.val)
    (hsym : IsSymmetric (toMatrix matrix n.val n.val h_len))
    (h_l : (alloc.vec.Vec.deref result.l).length = n.val * n.val) :
    let L := toMatrix (alloc.vec.Vec.deref result.l) n.val n.val h_l
    (∀ i j, i.val < j.val → L i j = 0) ∧
    (toMatrix matrix n.val n.val h_len) = L * Lᵀ :=
by
  sorry
