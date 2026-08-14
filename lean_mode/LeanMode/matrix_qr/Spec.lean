import LeanMode.matrix_qr.MatrixQr
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_qr
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: QR Decomposition

`qr_decomposition` factorises an m×n matrix A (column-major) via Householder
reflections, returning `Qr { q, r }` where:

> **A = Q · R**

with:
* `Q` — m×k matrix with orthonormal columns (`Qᵀ · Q = I_k`),
* `R` — k×n upper triangular matrix (`R[i][j] = 0` for `i > j`),
* `k = min(m, n)`.
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

/-! ### correctness -/

/-- **QR decomposition correctness.**

    The returned `q` (m×k) and `r` (k×n), viewed as matrices via `toMatrix`,
    satisfy `A = Q · R`, `Qᵀ · Q = I_k`, and `R` is upper triangular. -/
theorem qr_decomposition_correct
    (matrix : Slice ℝ) (nrows ncols : Std.Usize) (qr_result : Qr)
    (h : qr_decomposition matrix nrows ncols = ok qr_result)
    (h_len : matrix.length = nrows.val * ncols.val)
    (h_q : (alloc.vec.Vec.deref qr_result.q).length =
           nrows.val * (min nrows.val ncols.val))
    (h_r : (alloc.vec.Vec.deref qr_result.r).length =
           (min nrows.val ncols.val) * ncols.val) :
    let k := min nrows.val ncols.val
    let Q := toMatrix (alloc.vec.Vec.deref qr_result.q) nrows.val k h_q
    let R := toMatrix (alloc.vec.Vec.deref qr_result.r) k ncols.val h_r
    Qᵀ * Q = 1 ∧
    (∀ i j, j.val < i.val → R i j = 0) ∧
    (toMatrix matrix nrows.val ncols.val h_len) = Q * R :=
by
  sorry
