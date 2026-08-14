import LeanMode.matrix_eigen.MatrixEigen
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_eigen
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Symmetric Eigendecomposition

`symmetric_eigen` factorises a real **symmetric** n×n matrix A (row-major) via
cyclic Jacobi rotations into `Eigen { values, vectors }` where:

> **A = V · Λ · Vᵀ**

with:
* `V` — n×n orthogonal matrix whose columns are eigenvectors (`Vᵀ · V = I`),
* `Λ` — n×n diagonal matrix holding the (real) eigenvalues on its diagonal,
* column `j` of `V` is the eigenvector associated with `values[j]`.

The Jacobi method only diagonalizes symmetric matrices (real eigenvalues), so
symmetry of the input is a **precondition**, carried explicitly as the
`IsSymmetric` hypothesis of `symmetric_eigen_correct`.  The Rust function also
`assert!`s symmetry up front, so non-symmetric input is rejected (`fail`) at
run time rather than silently producing a wrong result.
-/

/-- Symmetry of a square matrix, expressed on the `toMatrix` view. -/
def IsSymmetric {n : Nat} (a : Matrix (Fin n) (Fin n) ℝ) : Prop :=
  ∀ i j, a i j = a j i

/-! ### helpers -/

/-- Row-major flat `Slice ℝ` → `Matrix (Fin rows) (Fin cols) ℝ`.
    Index: `element (row, col)` is at `row * cols + col`. -/
def toMatrix (s : Slice ℝ) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) ℝ :=
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

/-- Diagonal matrix built from a flat `Slice ℝ` of length `k`.
    Entry `(i, j)` is `sv[i]` on the diagonal and `0` off it. -/
def diagMatrix (sv : Slice ℝ) (k : Nat)
    (h : sv.length = k) : Matrix (Fin k) (Fin k) ℝ :=
  λ i j =>
    if i = j then
      have h_bound : i.val < sv.length := by
        rw [h]
        exact i.is_lt
      sv[i.val]'h_bound
    else 0

/-! ### correctness -/

/-- **Eigendecomposition output lengths.**

    The returned `values` and `vectors` have the correct lengths:
    `values` is n, and `vectors` is n×n. -/
theorem symmetric_eigen_length
    (matrix : Slice ℝ) (n : Std.Usize) (eps : ℝ) (max_sweeps : Std.Usize)
    (eigen_result : Eigen)
    (h : symmetric_eigen matrix n eps max_sweeps = ok eigen_result) :
    (alloc.vec.Vec.deref eigen_result.values).length = n.val ∧
    (alloc.vec.Vec.deref eigen_result.vectors).length = n.val * n.val :=
by
  sorry

/-- **Symmetric eigendecomposition correctness.**

    If `matrix` is a symmetric n×n matrix (`hsym`) and
    `symmetric_eigen matrix n eps max_sweeps` returns `ok eigen_result`, then the
    returned `vectors` (n×n) and `values` (n), viewed as matrices via
    `toMatrix` / `diagMatrix`, satisfy `A = V · Λ · Vᵀ`, with `Vᵀ · V = I`
    (orthonormal columns) and `Λ` diagonal. -/
theorem symmetric_eigen_correct
    (matrix : Slice ℝ) (n : Std.Usize) (eps : ℝ) (max_sweeps : Std.Usize)
    (eigen_result : Eigen)
    (h : symmetric_eigen matrix n eps max_sweeps = ok eigen_result)
    (h_len : matrix.length = n.val * n.val)
    (hsym : IsSymmetric (toMatrix matrix n.val n.val h_len))
    (h_values : (alloc.vec.Vec.deref eigen_result.values).length = n.val)
    (h_vectors : (alloc.vec.Vec.deref eigen_result.vectors).length = n.val * n.val) :
    let V := toMatrix (alloc.vec.Vec.deref eigen_result.vectors) n.val n.val h_vectors
    let L := diagMatrix (alloc.vec.Vec.deref eigen_result.values) n.val h_values
    Vᵀ * V = 1 ∧
    (∀ i j, i ≠ j → L i j = 0) ∧
    (toMatrix matrix n.val n.val h_len) = V * L * Vᵀ :=
by
  sorry
