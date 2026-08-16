import LeanMode.matrix_symmetric_eigen.MatrixSymmetricEigen
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_symmetric_eigen
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Symmetric Eigendecomposition

`symmetric_eigen` factorises a real **symmetric** n×n matrix A (column-major)
via Householder tridiagonalization + implicit QL iteration into
`SymmetricEigen { eigenvalues, eigenvectors }` where:

> **A = V · Λ · Vᵀ**

with:
* `V` — n×n orthogonal matrix whose columns are eigenvectors (`Vᵀ · V = I`),
* `Λ` — n×n diagonal matrix holding the (real) eigenvalues on its diagonal,
* column `j` of `V` is the eigenvector associated with `eigenvalues[j]`.

The eigenvalues of a symmetric matrix are all real, so symmetry of the input is
a **precondition**, carried explicitly as the `IsSymmetric` hypothesis below.
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

    The returned `eigenvalues` and `eigenvectors` have the correct lengths:
    `eigenvalues` is n, and `eigenvectors` is n×n. -/
theorem symmetric_eigen_length
    (matrix : Slice ℝ) (n : Std.Usize) (eps : ℝ) (max_niter : Std.Usize)
    (eigen_result : SymmetricEigen)
    (h : symmetric_eigen matrix n eps max_niter = ok eigen_result) :
    (alloc.vec.Vec.deref eigen_result.eigenvalues).length = n.val ∧
    (alloc.vec.Vec.deref eigen_result.eigenvectors).length = n.val * n.val :=
by
  sorry

/-- **Symmetric eigendecomposition correctness.**

    If `matrix` is a symmetric n×n matrix (`hsym`) and
    `symmetric_eigen matrix n eps max_niter` returns `ok eigen_result`, then the
    returned `eigenvectors` (n×n) and `eigenvalues` (n), viewed as matrices via
    `toMatrix` / `diagMatrix`, satisfy `A = V · Λ · Vᵀ`, with `Vᵀ · V = I`
    (orthonormal columns) and `Λ` diagonal. -/
theorem symmetric_eigen_correct
    (matrix : Slice ℝ) (n : Std.Usize) (eps : ℝ) (max_niter : Std.Usize)
    (eigen_result : SymmetricEigen)
    (h : symmetric_eigen matrix n eps max_niter = ok eigen_result)
    (h_len : matrix.length = n.val * n.val)
    (hsym : IsSymmetric (toMatrix matrix n.val n.val h_len))
    (h_values : (alloc.vec.Vec.deref eigen_result.eigenvalues).length = n.val)
    (h_vectors : (alloc.vec.Vec.deref eigen_result.eigenvectors).length = n.val * n.val) :
    let V := toMatrix (alloc.vec.Vec.deref eigen_result.eigenvectors) n.val n.val h_vectors
    let L := diagMatrix (alloc.vec.Vec.deref eigen_result.eigenvalues) n.val h_values
    Vᵀ * V = 1 ∧
    (∀ i j, i ≠ j → L i j = 0) ∧
    (toMatrix matrix n.val n.val h_len) = V * L * Vᵀ :=
by
  sorry
