import LeanMode.matrix_sparse_cholesky.MatrixSparseCholesky
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_sparse_cholesky
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Sparse Cholesky Factorization

`sparse_cholesky` factorises a symmetric positive-definite n×n matrix `A`,
given in CSR form, as

> **A = L · Lᵀ**

with `L` — n×n lower-triangular matrix with a positive diagonal.  The matrix
`L` is returned in CSR form; `csrDecode` reconstructs its dense view.

The input must be **symmetric** (`IsSymmetric` hypothesis below); positive-
definiteness is *not* carried as a hypothesis — the Rust `assert!` rejects a
non-positive diagonal pivot, so an `ok` result already implies the matrix was
positive-definite.
-/

/-! ### helpers -/

/-- Symmetry of a square matrix. -/
def IsSymmetric {n : Nat} (a : Matrix (Fin n) (Fin n) ℝ) : Prop :=
  ∀ i j, a i j = a j i

/-- Lower-triangularity: entries strictly above the diagonal are zero. -/
def IsLowerTriangular {n : Nat} (l : Matrix (Fin n) (Fin n) ℝ) : Prop :=
  ∀ i j, i.val < j.val → l i j = 0

/-- Strictly positive diagonal. -/
def HasPositiveDiagonal {n : Nat} (l : Matrix (Fin n) (Fin n) ℝ) : Prop :=
  ∀ i, 0 < l i i

/-- Decode a CSR (Compressed Sparse Row) matrix into a dense `Matrix`.
    `values[p]` is stored at `(i, j)` iff `row_ptr[i] ≤ p < row_ptr[i+1]` and
    `col_idx[p] = j`; every other entry is `0`.  (A valid CSR stores each
    `(i, j)` at most once, so the `∑` collapses to that single value.) -/
noncomputable def csrDecode
    (values : Slice ℝ) (col_idx : Slice Std.Usize) (row_ptr : Slice Std.Usize)
    (n : Nat) (h_rp : row_ptr.length = n + 1)
    (h_len : values.length = col_idx.length) :
    Matrix (Fin n) (Fin n) ℝ := by
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

/-! ### correctness -/

/-- **Sparse Cholesky correctness.**

    If the input CSR is well-formed (`h_uniq`: each `(i, j)` stored at most
    once), the decoded matrix `a` is symmetric (`hsym`), and
    `sparse_cholesky a_values a_col_idx a_row_ptr n` returns `ok result`, then
    the returned CSR `L`, decoded via `csrDecode`, is lower-triangular, has a
    positive diagonal, and satisfies `A = L · Lᵀ`. -/
theorem sparse_cholesky_correct
    (a_values : Slice ℝ) (a_col_idx : Slice Std.Usize) (a_row_ptr : Slice Std.Usize)
    (n : Std.Usize) (result : SparseCholesky)
    (h : sparse_cholesky a_values a_col_idx a_row_ptr n = ok result)
    (h_rp : a_row_ptr.length = n.val + 1)
    (h_len : a_values.length = a_col_idx.length)
    (h_uniq : CsrUnique a_col_idx a_row_ptr n.val h_rp)
    (hsym : IsSymmetric (csrDecode a_values a_col_idx a_row_ptr n.val h_rp h_len))
    (h_l_rp : (alloc.vec.Vec.deref result.l_row_ptr).length = n.val + 1)
    (h_l_len : (alloc.vec.Vec.deref result.l_values).length =
               (alloc.vec.Vec.deref result.l_col_idx).length) :
    let L := csrDecode (alloc.vec.Vec.deref result.l_values)
                       (alloc.vec.Vec.deref result.l_col_idx)
                       (alloc.vec.Vec.deref result.l_row_ptr) n.val h_l_rp h_l_len
    IsLowerTriangular L ∧
    HasPositiveDiagonal L ∧
    csrDecode a_values a_col_idx a_row_ptr n.val h_rp h_len = L * Lᵀ := by
  sorry
