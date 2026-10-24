import LeanMode.matrix_sparse_lu.MatrixSparseLu
import Mathlib.LinearAlgebra.Matrix.Determinant.Basic
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open Matrix
open matrix_sparse_lu

set_option linter.unusedVariables false

/-!
# Specification: Sparse LU Factorization

`sparse_lu` factorises an n×n matrix `A`, given in CSR form, with partial
(row) pivoting into

> **P · A = L · U**

where:
* `L` — n×n unit lower-triangular matrix (CSR),
* `U` — n×n upper-triangular matrix (CSR),
* `P` — permutation matrix encoded by `perm`.

Singular matrices cause `massert` to fail, so an `ok` result proves the input
was invertible.  `csrDecode` reconstructs the dense view of the CSR factors.
-/

/-! ### helpers -/

/-- Decode a CSR (Compressed Sparse Row) matrix into a dense `Matrix`. -/
noncomputable def csrDecode
    (values : Slice Rat) (col_idx : Slice Std.Usize) (row_ptr : Slice Std.Usize)
    (n : Nat) (h_rp : row_ptr.length = n + 1)
    (h_len : values.length = col_idx.length) :
    Matrix (Fin n) (Fin n) Rat := by
  intro i j
  have h_lo : i.val < row_ptr.length := by
    rw [h_rp]
    exact Nat.lt_trans i.is_lt (Nat.lt_succ_self n)
  have h_hi : i.val + 1 < row_ptr.length := by
    rw [h_rp]
    exact Nat.succ_lt_succ i.is_lt
  exact ∑ p : Fin values.length,
    if (row_ptr[i.val]'h_lo).val ≤ p.val ∧
       p.val < (row_ptr[i.val + 1]'h_hi).val ∧
       (col_idx[p.val]'(lt_of_lt_of_eq p.is_lt h_len)).val = j.val
    then values[p.val]'p.is_lt else 0

/-- CSR well-formedness: within every row, the stored column indices are
    pairwise distinct, i.e. each `(i, j)` is stored at most once.  This makes
    `csrDecode`'s `∑` over a row collapse to the single stored value, matching
    the implementation's overwrite semantics. -/
def CsrUnique (col_idx : Slice Std.Usize) (row_ptr : Slice Std.Usize) (n : Nat)
    (h_rp : row_ptr.length = n + 1) : Prop :=
  ∀ (i : Fin n) (p q : Fin col_idx.length),
    have h_lo : i.val < row_ptr.length := by
      rw [h_rp]
      exact Nat.lt_trans i.is_lt (Nat.lt_succ_self n)
    have h_hi : i.val + 1 < row_ptr.length := by
      rw [h_rp]
      exact Nat.succ_lt_succ i.is_lt
    (row_ptr[i.val]'h_lo).val ≤ p.val →
    p.val < (row_ptr[i.val + 1]'h_hi).val →
    (row_ptr[i.val]'h_lo).val ≤ q.val →
    q.val < (row_ptr[i.val + 1]'h_hi).val →
    p.val ≠ q.val →
    (col_idx[p.val]'p.is_lt).val ≠ (col_idx[q.val]'q.is_lt).val

/-- Unit lower-triangular: diagonal all 1, strictly upper part zero. -/
def IsUnitLowerTriangular {n : Nat} (l : Matrix (Fin n) (Fin n) Rat) : Prop :=
  (∀ i, l i i = 1) ∧ (∀ i j, i.val < j.val → l i j = 0)

/-- Upper-triangular: strictly lower part zero. -/
def IsUpperTriangular {n : Nat} (u : Matrix (Fin n) (Fin n) Rat) : Prop :=
  ∀ i j, j.val < i.val → u i j = 0

/-- Permutation vector `Vec Usize` → 0–1 permutation matrix.
    `P[i][j] = 1` iff `perm_vec[i] = j`. -/
def permMatrix (perm_vec : alloc.vec.Vec Std.Usize)
    (h_len : (alloc.vec.Vec.deref perm_vec).val.length = n) :
    Matrix (Fin n) (Fin n) Rat :=
  λ i j =>
    let lst := (alloc.vec.Vec.deref perm_vec).val
    have h_bound : i.val < lst.length := by
      have h_lst_len : lst.length = n := h_len
      rw [h_lst_len]
      exact i.is_lt
    let k := (lst.get ⟨i.val, h_bound⟩).val
    let p : Fin n := if h : k < n then ⟨k, h⟩ else i
    if p = j then 1 else 0

/-! ### correctness -/

/-- **Sparse LU factorization correctness.**

    If the input CSR is well-formed (`h_uniq`: each `(i, j)` stored at most
    once) and `sparse_lu a_values a_col_idx a_row_ptr n` returns `ok result`,
    then the decoded CSR factors `L` and `U` are respectively unit
    lower-triangular and upper-triangular, and satisfy `P · A = L · U` for the
    permutation matrix `P` encoded by `result.perm`. -/
theorem sparse_lu_correct
    (a_values : Slice Rat) (a_col_idx : Slice Std.Usize) (a_row_ptr : Slice Std.Usize)
    (n : Std.Usize) (result : SparseLU)
    (h : sparse_lu a_values a_col_idx a_row_ptr n = ok result)
    (h_rp : a_row_ptr.length = n.val + 1)
    (h_len : a_values.length = a_col_idx.length)
    (h_uniq : CsrUnique a_col_idx a_row_ptr n.val h_rp)
    (h_l_rp : (alloc.vec.Vec.deref result.l_row_ptr).length = n.val + 1)
    (h_l_len : (alloc.vec.Vec.deref result.l_values).length =
               (alloc.vec.Vec.deref result.l_col_idx).length)
    (h_u_rp : (alloc.vec.Vec.deref result.u_row_ptr).length = n.val + 1)
    (h_u_len : (alloc.vec.Vec.deref result.u_values).length =
               (alloc.vec.Vec.deref result.u_col_idx).length)
    (h_perm_len : (alloc.vec.Vec.deref result.perm).val.length = n.val) :
    let L := csrDecode (alloc.vec.Vec.deref result.l_values)
                       (alloc.vec.Vec.deref result.l_col_idx)
                       (alloc.vec.Vec.deref result.l_row_ptr) n.val h_l_rp h_l_len
    let U := csrDecode (alloc.vec.Vec.deref result.u_values)
                       (alloc.vec.Vec.deref result.u_col_idx)
                       (alloc.vec.Vec.deref result.u_row_ptr) n.val h_u_rp h_u_len
    IsUnitLowerTriangular L ∧
    IsUpperTriangular U ∧
    (permMatrix result.perm h_perm_len) *
      (csrDecode a_values a_col_idx a_row_ptr n.val h_rp h_len) = L * U := by
  sorry
