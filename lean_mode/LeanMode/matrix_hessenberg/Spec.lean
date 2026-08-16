import LeanMode.matrix_hessenberg.MatrixHessenberg
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_hessenberg
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Hessenberg Decomposition

`hessenberg_decomposition` reduces an n×n matrix A (column-major) to
Hessenberg form via Householder reflections, returning `Hessenberg { q, h }`
where:

> **A = Q · H · Qᵀ**

with:
* `Q` — n×n orthogonal matrix (`Qᵀ · Q = Q · Qᵀ = I`),
* `H` — n×n upper Hessenberg matrix (`H[i][j] = 0` for `i > j + 1`).
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

/-- **Hessenberg decomposition correctness.**

    The returned `q` (n×n) and `h` (n×n), viewed as matrices via `toMatrix`,
    satisfy `A = Q · H · Qᵀ`, `Q` orthogonal, and `H` upper Hessenberg. -/
theorem hessenberg_decomposition_correct
    (matrix : Slice ℝ) (n : Std.Usize) (result : Hessenberg)
    (h : hessenberg_decomposition matrix n = ok result)
    (h_len : matrix.length = n.val * n.val)
    (h_q : (alloc.vec.Vec.deref result.q).length = n.val * n.val)
    (h_h : (alloc.vec.Vec.deref result.h).length = n.val * n.val) :
    let A := toMatrix matrix n.val n.val h_len
    let Q := toMatrix (alloc.vec.Vec.deref result.q) n.val n.val h_q
    let H := toMatrix (alloc.vec.Vec.deref result.h) n.val n.val h_h
    Qᵀ * Q = 1 ∧
    Q * Qᵀ = 1 ∧
    (∀ i j, j.val + 1 < i.val → H i j = 0) ∧
    A = Q * H * Qᵀ :=
by
  sorry
