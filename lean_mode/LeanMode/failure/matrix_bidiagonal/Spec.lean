import LeanMode.matrix_bidiagonal.MatrixBidiagonal
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_bidiagonal
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Bidiagonal Decomposition

`bidiagonal` decomposes a general m×n matrix M (column-major) using
Householder reflections, returning the three factors (U, B, Vt) such that

> **M = U · B · Vt**

where:
* `U` — m×k matrix with orthonormal columns, k = min(m, n)
* `B` — k×k bidiagonal matrix (nonzero on diagonal and first super/sub-diagonal)
* `Vt` — k×n matrix with orthonormal rows (Vᵀ above)

All matrices are stored in **column-major** order.

Precondition: m > 0 and n > 0 (panics otherwise).
-/

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

/-! ### orthogonality -/

/-- A matrix U (m×k) has orthonormal columns iff `Uᵀ · U = I_k`. -/
def HasOrthonormalCols {m k : Nat} (U : Matrix (Fin m) (Fin k) ℝ) : Prop :=
  Uᵀ * U = 1

/-- A matrix Vt (k×n) has orthonormal rows iff `Vt · Vtᵀ = I_k`. -/
def HasOrthonormalRows {k n : Nat} (Vt : Matrix (Fin k) (Fin n) ℝ) : Prop :=
  Vt * Vtᵀ = 1

/-! ### correctness — dimensions -/

/-- The returned U, B, Vt have the correct dimensions:
    U is m×k, B is k×k, Vt is k×n where k = min(m,n). -/
theorem bidiagonal_dimensions
    (matrix : Slice ℝ) (nrows ncols : Std.Usize)
    (U_vec B_vec Vt_vec : alloc.vec.Vec ℝ) (modified : Slice ℝ)
    (h : bidiagonal matrix nrows ncols =
         ok ((U_vec, B_vec, Vt_vec), modified))
    (h_nonempty : nrows.val > 0 ∧ ncols.val > 0) :
    let k := min nrows.val ncols.val
    (alloc.vec.Vec.deref U_vec).val.length = nrows.val * k ∧
    (alloc.vec.Vec.deref B_vec).val.length = k * k ∧
    (alloc.vec.Vec.deref Vt_vec).val.length = k * ncols.val :=
by
  sorry

/-! ### correctness — orthogonality -/

/-- **U has orthonormal columns, Vt has orthonormal rows.**

    U is m×k with `Uᵀ·U = I_k`.  Vt is k×n with `Vt·Vtᵀ = I_k`. -/
theorem bidiagonal_orthogonal
    (matrix : Slice ℝ) (nrows ncols : Std.Usize)
    (U_vec B_vec Vt_vec : alloc.vec.Vec ℝ) (modified : Slice ℝ)
    (h : bidiagonal matrix nrows ncols =
         ok ((U_vec, B_vec, Vt_vec), modified))
    (h_nonempty : nrows.val > 0 ∧ ncols.val > 0)
    (h_u : (alloc.vec.Vec.deref U_vec).val.length = nrows.val * min nrows.val ncols.val)
    (h_vt : (alloc.vec.Vec.deref Vt_vec).val.length = (min nrows.val ncols.val) * ncols.val) :
    HasOrthonormalCols (toMatrix (alloc.vec.Vec.deref U_vec) nrows.val (min nrows.val ncols.val) h_u) ∧
    HasOrthonormalRows (toMatrix (alloc.vec.Vec.deref Vt_vec) (min nrows.val ncols.val) ncols.val h_vt) :=
by
  sorry

/-! ### correctness — decomposition -/

/-- **Bidiagonal decomposition correctness.**

    M = U · B · Vt, where B is bidiagonal (nonzero only on the diagonal
    and first super-diagonal when m ≥ n, or first sub-diagonal when m < n). -/
theorem bidiagonal_decomposition
    (matrix : Slice ℝ) (nrows ncols : Std.Usize)
    (U_vec B_vec Vt_vec : alloc.vec.Vec ℝ) (modified : Slice ℝ)
    (h : bidiagonal matrix nrows ncols =
         ok ((U_vec, B_vec, Vt_vec), modified))
    (h_nonempty : nrows.val > 0 ∧ ncols.val > 0)
    (h_sizes : matrix.length = nrows.val * ncols.val)
    (h_u : (alloc.vec.Vec.deref U_vec).val.length = nrows.val * min nrows.val ncols.val)
    (h_b : (alloc.vec.Vec.deref B_vec).val.length = (min nrows.val ncols.val) * (min nrows.val ncols.val))
    (h_vt : (alloc.vec.Vec.deref Vt_vec).val.length = (min nrows.val ncols.val) * ncols.val) :
    let k := min nrows.val ncols.val
    toMatrix matrix nrows.val ncols.val h_sizes =
      (toMatrix (alloc.vec.Vec.deref U_vec) nrows.val k h_u) *
      (toMatrix (alloc.vec.Vec.deref B_vec) k k h_b) *
      (toMatrix (alloc.vec.Vec.deref Vt_vec) k ncols.val h_vt) :=
by
  sorry
