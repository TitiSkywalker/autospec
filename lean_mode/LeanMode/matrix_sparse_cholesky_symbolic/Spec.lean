import LeanMode.matrix_sparse_cholesky_symbolic.MatrixSparseCholeskySymbolic
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_sparse_cholesky_symbolic

set_option linter.unusedVariables false

/-!
# Specification: Symbolic Sparse Cholesky Factorization and the Elimination Tree

`symbolic_factorization col_idx row_ptr n` reads the *sparsity pattern* of a
symmetric `n × n` matrix `A` (full symmetric, CSR) and returns

* the CSC pattern of the Cholesky factor `L` — its **fill-in** included, and
* the **elimination tree** of that pattern.

The mathematical content is the *fill path theorem* of Rose–Tarjan–Lueker:

> for `i > j`, the entry `L[i][j]` is structurally non-zero if and only if
> `A[i][j] ≠ 0`, or there is a path `i = v₀, v₁, …, v_k = j` in the graph of
> `A` whose interior vertices `v₁, …, v_{k-1}` are all smaller than `j`.

(`FillPath` below.)  Equivalently: eliminating the vertices `0, 1, …, j-1` makes
`i` and `j` adjacent.  The elimination tree is then
`parent[j] = min { i > j : L[i][j] ≠ 0 }` (with `n` marking a root), and the
specification states both, plus that the computed pattern is lower triangular
with a full diagonal.

Note the interior-vertices bound is `< j` and *not* "`j` is an ancestor of `i`
in the elimination tree": the two differ (in the 4-cycle `0—1—2—3—0`, the fill
entry `L[3][1]` exists while `1` is not an ancestor of `3`).
-/

/-! ### helpers: reading the CSR/CSC arrays -/

/-- `row_ptr[i]` as a `Nat`. -/
def rowPtr (row_ptr : Slice Std.Usize) (n : Nat) (h_rp : row_ptr.length = n + 1)
    (i : Fin n) : Nat :=
  (row_ptr[i.val]'(by
    show i.val < row_ptr.length
    rw [h_rp]
    exact Nat.lt_trans i.is_lt (Nat.lt_succ_self n))).val

/-- `row_ptr[i + 1]` as a `Nat`. -/
def rowPtrNext (row_ptr : Slice Std.Usize) (n : Nat) (h_rp : row_ptr.length = n + 1)
    (i : Fin n) : Nat :=
  (row_ptr[i.val + 1]'(by
    show i.val + 1 < row_ptr.length
    rw [h_rp]
    exact Nat.succ_lt_succ i.is_lt)).val

/-- `col_ptr[j]` as a `Nat`. -/
def colPtr (col_ptr : Slice Std.Usize) (n : Nat) (h_cp : col_ptr.length = n + 1)
    (j : Fin n) : Nat :=
  (col_ptr[j.val]'(by
    show j.val < col_ptr.length
    rw [h_cp]
    exact Nat.lt_trans j.is_lt (Nat.lt_succ_self n))).val

/-- `col_ptr[j + 1]` as a `Nat`. -/
def colPtrNext (col_ptr : Slice Std.Usize) (n : Nat) (h_cp : col_ptr.length = n + 1)
    (j : Fin n) : Nat :=
  (col_ptr[j.val + 1]'(by
    show j.val + 1 < col_ptr.length
    rw [h_cp]
    exact Nat.succ_lt_succ j.is_lt)).val

/-- The `j`-th entry of the elimination tree, as a `Nat`. -/
def parAt (par : Slice Std.Usize) (n : Nat) (h_par : par.length = n)
    (j : Fin n) : Nat :=
  (par[j.val]'(by
    show j.val < par.length
    rw [h_par]
    exact j.is_lt)).val

/-- The CSR pattern of the input has an entry at `(i, j)`. -/
def Edge (col_idx row_ptr : Slice Std.Usize) (n : Nat)
    (h_rp : row_ptr.length = n + 1) (i j : Fin n) : Prop :=
  ∃ p : Fin col_idx.length,
    rowPtr row_ptr n h_rp i ≤ p.val ∧ p.val < rowPtrNext row_ptr n h_rp i ∧
    (col_idx[p.val]'p.is_lt).val = j.val

/-- Row `i` is present in column `j` of the computed pattern of `L`. -/
def LPresent (l_row_idx l_col_ptr : Slice Std.Usize) (n : Nat)
    (h_cp : l_col_ptr.length = n + 1) (i j : Fin n) : Prop :=
  ∃ q : Fin l_row_idx.length,
    colPtr l_col_ptr n h_cp j ≤ q.val ∧ q.val < colPtrNext l_col_ptr n h_cp j ∧
    (l_row_idx[q.val]'q.is_lt).val = i.val

/-- One edge of a path that stays strictly below `j`. -/
def StepBelow (col_idx row_ptr : Slice Std.Usize) (n : Nat)
    (h_rp : row_ptr.length = n + 1) (j : Fin n) (u v : Fin n) : Prop :=
  u.val < j.val ∧ v.val < j.val ∧ Edge col_idx row_ptr n h_rp u v

/-- **The fill path**: `i` reaches `j` in the graph of `A` through interior
    vertices that are all smaller than `j` (Rose–Tarjan–Lueker).  The `k = 1`
    case — the direct edge `A[i][j] ≠ 0` — is the first disjunct; in the second,
    `v` is the first and `w` the last interior vertex of the path, and the
    segment between them lies entirely below `j`. -/
def FillPath (col_idx row_ptr : Slice Std.Usize) (n : Nat)
    (h_rp : row_ptr.length = n + 1) (i j : Fin n) : Prop :=
  Edge col_idx row_ptr n h_rp i j ∨
  ∃ v w : Fin n,
    v.val < j.val ∧ w.val < j.val ∧
    Edge col_idx row_ptr n h_rp i v ∧
    Relation.ReflTransGen (StepBelow col_idx row_ptr n h_rp j) v w ∧
    Edge col_idx row_ptr n h_rp w j

/-! ### correctness -/

/-- **Symbolic factorization correctness** (the fill path theorem + the
    elimination tree).

    Hypotheses: the CSR input is well-formed (`h_rp0`, `h_rpm` — the row
    pointers are non-decreasing — and `h_rplast`) and its pattern is symmetric
    (`h_symm`).  The conclusion: the diagonal of the pattern of `L` is full,
    above the diagonal the pattern is exactly the fill, the pattern is lower
    triangular, and `parent[j]` is the smallest row index `> j` present in
    column `j` of `L` (the sentinel `n` when there is none). -/
theorem symbolic_factorization_correct
    (a_col_idx a_row_ptr : Slice Std.Usize) (n : Std.Usize)
    (result : SymbolicFactorization)
    (h_rp : a_row_ptr.length = n.val + 1)
    (h_rp0 : (a_row_ptr[0]'(by
      show 0 < a_row_ptr.length
      rw [h_rp]
      exact Nat.succ_pos n.val)).val = 0)
    (h_rpm : ∀ i : Fin n.val, rowPtr a_row_ptr n.val h_rp i ≤ rowPtrNext a_row_ptr n.val h_rp i)
    (h_rplast : (a_row_ptr[n.val]'(by
      show n.val < a_row_ptr.length
      rw [h_rp]
      exact Nat.lt_succ_self n.val)).val = a_col_idx.length)
    (h_symm : ∀ i j : Fin n.val,
      Edge a_col_idx a_row_ptr n.val h_rp i j ↔ Edge a_col_idx a_row_ptr n.val h_rp j i)
    (h_cp : (alloc.vec.Vec.deref result.l_col_ptr).length = n.val + 1)
    (h_par : (alloc.vec.Vec.deref result.parent).length = n.val)
    (h : symbolic_factorization a_col_idx a_row_ptr n = ok result) :
    let C := alloc.vec.Vec.deref result.l_col_ptr
    let R := alloc.vec.Vec.deref result.l_row_idx
    let par := alloc.vec.Vec.deref result.parent
    -- the diagonal is always structurally non-zero
    (∀ i : Fin n.val, LPresent R C n.val h_cp i i) ∧
    -- above the diagonal, `L` is present exactly on the fill paths
    (∀ i j : Fin n.val, j.val < i.val →
      (LPresent R C n.val h_cp i j ↔ FillPath a_col_idx a_row_ptr n.val h_rp i j)) ∧
    -- nothing above the diagonal
    (∀ i j : Fin n.val, i.val < j.val → ¬ LPresent R C n.val h_cp i j) ∧
    -- the elimination tree: either column `j` of `L` has nothing below the
    -- diagonal and `parent[j]` is the sentinel `n`, or `parent[j]` is the
    -- smallest row index `> j` present in that column
    (∀ j : Fin n.val,
      (parAt par n.val h_par j = n.val ∧
        ∀ i : Fin n.val, j.val < i.val → ¬ LPresent R C n.val h_cp i j) ∨
      ∃ i : Fin n.val,
        parAt par n.val h_par j = i.val ∧ j.val < i.val ∧
        LPresent R C n.val h_cp i j ∧
        (∀ k : Fin n.val, j.val < k.val → k.val < i.val →
          ¬ LPresent R C n.val h_cp k j)) := by
  sorry
