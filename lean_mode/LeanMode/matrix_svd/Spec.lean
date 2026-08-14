import LeanMode.matrix_svd.MatrixSvd
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_svd
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Singular Value Decomposition

`svd` factorises an m×n matrix A (row-major) into `Svd { u, singular_values, v_t }`
where:

> **A = U · Σ · Vᵀ**

with:
* `U` — m×k matrix with orthonormal columns (`Uᵀ · U = I_k`),
* `Σ` — k×k diagonal matrix with non-negative entries,
* `Vᵀ` — k×n matrix with orthonormal rows (`Vᵀ · V = I_k`),
* `k = min(m, n)`.
-/

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

/-- **SVD output lengths.**

    The returned `u`, `singular_values` and `v_t` have the correct lengths:
    `u` is m×k, `singular_values` is k, and `v_t` is k×n, where k = min(m, n). -/
theorem svd_length
    (matrix : Slice ℝ) (nrows ncols : Std.Usize) (eps : ℝ) (max_sweeps : Std.Usize)
    (svd_result : Svd)
    (h : svd matrix nrows ncols eps max_sweeps = ok svd_result) :
    (alloc.vec.Vec.deref svd_result.u).length = nrows.val * (min nrows.val ncols.val) ∧
    (alloc.vec.Vec.deref svd_result.singular_values).length = min nrows.val ncols.val ∧
    (alloc.vec.Vec.deref svd_result.v_t).length = (min nrows.val ncols.val) * ncols.val :=
by
  sorry

/-- **Singular value decomposition correctness.**

    The returned `u` (m×k), `singular_values` (k) and `v_t` (k×n), viewed as
    matrices via `toMatrix` / `diagMatrix`, satisfy `A = U · Σ · Vᵀ`, with
    `Uᵀ · U = I_k`, `Vᵀ · V = I_k`, and `Σ` diagonal with non-negative entries. -/
theorem svd_correct
    (matrix : Slice ℝ) (nrows ncols : Std.Usize) (eps : ℝ) (max_sweeps : Std.Usize)
    (svd_result : Svd)
    (h : svd matrix nrows ncols eps max_sweeps = ok svd_result)
    (h_len : matrix.length = nrows.val * ncols.val)
    (h_u : (alloc.vec.Vec.deref svd_result.u).length =
           nrows.val * (min nrows.val ncols.val))
    (h_s : (alloc.vec.Vec.deref svd_result.singular_values).length =
           min nrows.val ncols.val)
    (h_vt : (alloc.vec.Vec.deref svd_result.v_t).length =
            (min nrows.val ncols.val) * ncols.val) :
    let k := min nrows.val ncols.val
    let U := toMatrix (alloc.vec.Vec.deref svd_result.u) nrows.val k h_u
    let S := diagMatrix (alloc.vec.Vec.deref svd_result.singular_values) k h_s
    let Vt := toMatrix (alloc.vec.Vec.deref svd_result.v_t) k ncols.val h_vt
    Uᵀ * U = 1 ∧
    Vt * Vtᵀ = 1 ∧
    (∀ i j, i ≠ j → S i j = 0) ∧
    (∀ i, 0 ≤ S i i) ∧
    (toMatrix matrix nrows.val ncols.val h_len) = U * S * Vt :=
by
  sorry
