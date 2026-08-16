import LeanMode.matrix_bidiagonal.MatrixBidiagonal
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_bidiagonal
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Bidiagonalization

`bidiagonal_decomposition` reduces an m×n matrix A (column-major, m ≥ n) to
upper bidiagonal form via Householder reflections, returning
`Bidiagonal { u, b, v_t }` where:

> **A = U · B · Vᵀ**

with:
* `U` — m×n matrix with orthonormal columns (`Uᵀ · U = I_n`),
* `B` — n×n upper bidiagonal matrix (`B[i][j] = 0` for `i > j` or `i + 1 < j`),
* `Vᵀ` — n×n matrix with orthonormal rows (`Vᵀ · V = I_n`).

The decomposition is the *upper* bidiagonal case, valid only when
`nrows ≥ ncols`; this precondition is stated explicitly as `h_ge` below (the
Rust `assert!` additionally guards it).
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

/-- **Bidiagonalization output lengths.**

    Under the precondition `nrows ≥ ncols`, the returned `u`, `b` and `v_t`
    have the correct lengths: `u` is m×n, and `b` / `v_t` are n×n. -/
theorem bidiagonal_length
    (matrix : Slice ℝ) (nrows ncols : Std.Usize) (result : Bidiagonal)
    (h_ge : nrows.val >= ncols.val)
    (h : bidiagonal_decomposition matrix nrows ncols = ok result) :
    (alloc.vec.Vec.deref result.u).length = nrows.val * ncols.val ∧
    (alloc.vec.Vec.deref result.b).length = ncols.val * ncols.val ∧
    (alloc.vec.Vec.deref result.v_t).length = ncols.val * ncols.val :=
by
  sorry

/-- **Bidiagonalization correctness.**

    Under the precondition `nrows ≥ ncols`, the returned `u` (m×n), `b` (n×n)
    and `v_t` (n×n), viewed as matrices via `toMatrix`, satisfy
    `A = U · B · Vᵀ`, with `Uᵀ · U = I`, `Vᵀ · V = I`, and `B` upper
    bidiagonal. -/
theorem bidiagonal_decomposition_correct
    (matrix : Slice ℝ) (nrows ncols : Std.Usize) (result : Bidiagonal)
    (h_ge : nrows.val >= ncols.val)
    (h : bidiagonal_decomposition matrix nrows ncols = ok result)
    (h_len : matrix.length = nrows.val * ncols.val)
    (h_u : (alloc.vec.Vec.deref result.u).length = nrows.val * ncols.val)
    (h_b : (alloc.vec.Vec.deref result.b).length = ncols.val * ncols.val)
    (h_v : (alloc.vec.Vec.deref result.v_t).length = ncols.val * ncols.val) :
    let A := toMatrix matrix nrows.val ncols.val h_len
    let U := toMatrix (alloc.vec.Vec.deref result.u) nrows.val ncols.val h_u
    let B := toMatrix (alloc.vec.Vec.deref result.b) ncols.val ncols.val h_b
    let Vt := toMatrix (alloc.vec.Vec.deref result.v_t) ncols.val ncols.val h_v
    Uᵀ * U = 1 ∧
    Vt * Vtᵀ = 1 ∧
    (∀ i j, j.val < i.val → B i j = 0) ∧
    (∀ i j, i.val + 1 < j.val → B i j = 0) ∧
    A = U * B * Vt :=
by
  sorry
