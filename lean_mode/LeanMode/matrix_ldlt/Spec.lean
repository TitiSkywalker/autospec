import LeanMode.matrix_ldlt.MatrixLdlt
import Mathlib.LinearAlgebra.Matrix.Determinant.Basic
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_ldlt

set_option linter.unusedVariables false

/-!
# Specification: LDLᵀ Decomposition

`ldlt_decomposition` factorises an n×n symmetric matrix A as
**A = L · D · Lᵀ**, where L is unit lower triangular and D is diagonal.

## Preconditions

The caller must provide a symmetric matrix (`Aᵀ = A`); the algorithm
reads only the lower triangle and implicitly assumes symmetry.

## When does it succeed?

The algorithm only requires that every leading principal minor is
**non-zero** — it does **not** require positive definiteness.  If some
`D[i]` turns out negative, the decomposition is still valid.

Positive definite matrices (all `D[i] > 0`) are the most common use
case, but indefinite matrices such as `[[1,2],[2,1]]` (determinant −3,
`D = diag(1, −3)`) are handled correctly as well.

The only failure case is a zero pivot (`diag = 0`), which signals a
singular matrix or one whose leading principal minor vanishes.
`massert` reports this, so an `ok` result proves all pivots were
non-zero.
-/

/-! ### helpers -/

/-- Flat `Slice Rat` (row-major) → `Matrix (Fin rows) (Fin cols) Rat`. -/
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

/-- Extract L (unit lower triangular) from the combined LDLᵀ matrix.
    L[i][i] = 1, L[i][j] = ld[i][j] for i > j, L[i][j] = 0 for i < j. -/
def extractL {n : Nat} (ld : Matrix (Fin n) (Fin n) Rat) :
    Matrix (Fin n) (Fin n) Rat :=
  λ i j => if i.val = j.val then 1 else if i.val > j.val then ld i j else 0

/-- Extract D (diagonal) from the combined LDLᵀ matrix.
    D[i][i] = ld[i][i], D[i][j] = 0 for i ≠ j. -/
def extractD {n : Nat} (ld : Matrix (Fin n) (Fin n) Rat) :
    Matrix (Fin n) (Fin n) Rat :=
  λ i j => if i.val = j.val then ld i j else 0

/-! ### structural properties -/

lemma extractL_unit_lower_triangular {n : Nat} (ld : Matrix (Fin n) (Fin n) Rat) :
    (∀ i, extractL ld i i = 1) ∧ (∀ i j, i.val < j.val → extractL ld i j = 0) := by
  constructor
  · intro i; unfold extractL; simp
  · intro i j h_lt
    unfold extractL
    have h_ne : i.val ≠ j.val := Nat.ne_of_lt h_lt
    have h_not_gt : ¬ i.val > j.val := Nat.not_lt.mpr (Nat.le_of_lt h_lt)
    simp [h_ne, h_not_gt]

lemma extractD_diagonal {n : Nat} (ld : Matrix (Fin n) (Fin n) Rat) :
    ∀ i j, i.val ≠ j.val → extractD ld i j = 0 := by
  intro i j h_ne
  unfold extractD
  simp [h_ne]

/-! ## correctness -/

theorem ldlt_decomposition_correct
    (matrix : Slice Rat) (n : Std.Usize) (result : Ldlt)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h_symm : (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len)ᵀ =
              toMatrix matrix (n.val : Nat) (n.val : Nat) h_len)
    (h : ldlt_decomposition matrix n = ok result)
    (h_ld_len : (alloc.vec.Vec.deref result.ld).val.length =
                (n.val : Nat) * (n.val : Nat)) :
    (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len) =
    (extractL (toMatrix (alloc.vec.Vec.deref result.ld)
                       (n.val : Nat) (n.val : Nat) h_ld_len)) *
    (extractD (toMatrix (alloc.vec.Vec.deref result.ld)
                       (n.val : Nat) (n.val : Nat) h_ld_len)) *
    (extractL (toMatrix (alloc.vec.Vec.deref result.ld)
                       (n.val : Nat) (n.val : Nat) h_ld_len))ᵀ := by
  sorry
