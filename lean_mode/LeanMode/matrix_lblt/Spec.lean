import LeanMode.matrix_lblt.MatrixLblt
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_lblt
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Bunch–Kaufman LBLᵀ Factorization

`lblt_decomposition` factorises a real symmetric n×n matrix A (column-major) via
Bunch–Kaufman block-diagonal pivoting into `Lblt { l, d }` where:

> **A = L · B · Lᵀ**

with:
* `L` — n×n permuted unit-lower-triangular factor (in general *not*
  lower-triangular in the original basis),
* `B` — n×n block-diagonal factor with 1×1 and 2×2 diagonal blocks (i.e.
  tridiagonal / bandwidth 1).

Unlike LDLᵀ, this also works for indefinite matrices (via the 2×2 pivot
blocks).  The input must be symmetric (`IsSymmetric` hypothesis below).
-/

/-- Symmetry of a square matrix, expressed on the `toMatrix` view. -/
def IsSymmetric {n : Nat} (a : Matrix (Fin n) (Fin n) Rat) : Prop :=
  ∀ i j, a i j = a j i

/-! ### helpers -/

/-- Column-major flat `Slice Rat` → `Matrix (Fin rows) (Fin cols) Rat`.
    Index: `element (row, col)` is at `row + col * nrows`. -/
def toMatrix (s : Slice Rat) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) Rat :=
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

/-- **LBLᵀ output lengths.**

    The returned `l` and `d` have the correct length n×n. -/
theorem lblt_length
    (matrix : Slice Rat) (n : Std.Usize) (result : Lblt)
    (h : lblt_decomposition matrix n = ok result) :
    (alloc.vec.Vec.deref result.l).length = n.val * n.val ∧
    (alloc.vec.Vec.deref result.d).length = n.val * n.val :=
by
  sorry

/-- **LBLᵀ factorization correctness.**

    If `matrix` is symmetric (`hsym`) and `lblt_decomposition matrix n` returns
    `ok result`, then the returned `l` (n×n) and `d` (n×n), viewed as matrices
    via `toMatrix`, satisfy `A = L · B · Lᵀ`, with `B` tridiagonal (bandwidth 1,
    i.e. block diagonal with 1×1/2×2 blocks). -/
theorem lblt_correct
    (matrix : Slice Rat) (n : Std.Usize) (result : Lblt)
    (h : lblt_decomposition matrix n = ok result)
    (h_m : matrix.length = n.val * n.val)
    (hsym : IsSymmetric (toMatrix matrix n.val n.val h_m))
    (h_l : (alloc.vec.Vec.deref result.l).length = n.val * n.val)
    (h_d : (alloc.vec.Vec.deref result.d).length = n.val * n.val) :
    let L := toMatrix (alloc.vec.Vec.deref result.l) n.val n.val h_l
    let B := toMatrix (alloc.vec.Vec.deref result.d) n.val n.val h_d
    (∀ i j, j.val + 1 < i.val → B i j = 0) ∧
    (∀ i j, i.val + 1 < j.val → B i j = 0) ∧
    (toMatrix matrix n.val n.val h_m) = L * B * Lᵀ :=
by
  sorry
