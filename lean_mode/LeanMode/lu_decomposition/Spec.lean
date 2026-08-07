import LeanMode.lu_decomposition.LuDecomposition
import Mathlib.LinearAlgebra.Matrix.Determinant.Basic
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open Matrix
open lu_decomposition

set_option linter.unusedVariables false

/-!
# Specification: LU Decomposition

`lu_decomposition` factorises an n×n matrix A via Gaussian elimination
with partial pivoting.  On success it returns `LU { lu, perm }` such that

> **P · A = L · U**

where L is unit lower triangular, U is upper triangular, and P is the
permutation matrix encoded by `perm`.  Singular matrices cause `massert`
to fail, so an `ok` result proves the input was invertible.
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

/-- Extract L (unit lower triangular) from the compact LU matrix. -/
def extractL {n : Nat} (lu : Matrix (Fin n) (Fin n) Rat) :
    Matrix (Fin n) (Fin n) Rat :=
  λ i j => if i.val = j.val then 1 else if i.val > j.val then lu i j else 0

/-- Extract U (upper triangular) from the compact LU matrix. -/
def extractU {n : Nat} (lu : Matrix (Fin n) (Fin n) Rat) :
    Matrix (Fin n) (Fin n) Rat :=
  λ i j => if i.val ≤ j.val then lu i j else 0

/-- Permutation vector `Vec Usize` → 0–1 permutation matrix.
    `P[i][j] = 1` iff `perm_vec[i] = j`. -/
def permMatrix (perm_vec : alloc.vec.Vec Std.Usize)
    (h_len : (alloc.vec.Vec.deref perm_vec).val.length = n) :
    Matrix (Fin n) (Fin n) Rat :=
  λ i j =>
    let lst := perm_vec.val
    have h_bound : i.val < lst.length := by
      have h_lst_len : lst.length = n := h_len
      rw [h_lst_len]
      exact i.is_lt
    let k := (lst.get ⟨i.val, h_bound⟩).val
    let p : Fin n := if h : k < n then ⟨k, h⟩ else i
    if p = j then 1 else 0

/-! ### structural properties -/

lemma extractL_unit_lower_triangular {n : Nat} (lu : Matrix (Fin n) (Fin n) Rat) :
    (∀ i, extractL lu i i = 1) ∧ (∀ i j, i.val < j.val → extractL lu i j = 0) := by
  constructor
  · intro i; unfold extractL; simp
  · intro i j h_lt
    unfold extractL
    have h_ne : i.val ≠ j.val := Nat.ne_of_lt h_lt
    have h_not_gt : ¬ i.val > j.val := Nat.not_lt.mpr (Nat.le_of_lt h_lt)
    simp [h_ne, h_not_gt]

lemma extractU_upper_triangular {n : Nat} (lu : Matrix (Fin n) (Fin n) Rat) :
    ∀ i j, j.val < i.val → extractU lu i j = 0 := by
  intro i j h_lt
  unfold extractU
  simp [Nat.not_le.mpr h_lt]

/-! ## correctness -/

theorem lu_decomposition_correct
    (matrix : Slice Rat) (n : Std.Usize) (lu : LU)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h : lu_decomposition matrix n = ok lu)
    (h_lu_len : (alloc.vec.Vec.deref lu.lu).val.length =
                (n.val : Nat) * (n.val : Nat))
    (h_perm_len : (alloc.vec.Vec.deref lu.perm).val.length = (n.val : Nat)) :
    (permMatrix lu.perm h_perm_len) *
      (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len) =
    (extractL (toMatrix (alloc.vec.Vec.deref lu.lu)
               (n.val : Nat) (n.val : Nat) h_lu_len)) *
    (extractU (toMatrix (alloc.vec.Vec.deref lu.lu)
               (n.val : Nat) (n.val : Nat) h_lu_len)) := by
  sorry
