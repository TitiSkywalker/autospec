import LeanMode.matrix_jordan.MatrixJordan
import Mathlib

open Aeneas Aeneas.Std Result
open Matrix
open matrix_jordan

set_option linter.unusedVariables false

/-!
# Specification: n×n Matrix Jordan Normal Form

`jordan_form` computes the Jordan normal form J of a square matrix A
(row-major) via:

1. the characteristic polynomial `det(x·I − A)` (Faddeev–LeVerrier),
2. factoring it over the integers (a monic integer characteristic polynomial
   has integer rational roots),
3. the dimensions of the generalized eigenspaces `ker((A − λ·I)^j)`, which
   determine the Jordan block sizes,
4. assembling the block-diagonal Jordan matrix.

The deep insight is the **generalized-eigenspace** decomposition: the number of
Jordan blocks of size ≥ j for an eigenvalue λ is
`dim ker((A−λI)^j) − dim ker((A−λI)^{j−1})`.  We state the main result as the
similarity `A = P·J·P⁻¹` together with the structural invariant that `J` is in
Jordan normal form.
-/

/-- Row-major flat `Slice Rat` → Mathlib `Matrix`.
    `(i, j) ↦ s[i * cols + j]` -/
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

/-- `J` is in Jordan normal form: upper triangular, only the diagonal and
    superdiagonal may be nonzero, every superdiagonal entry is `0` or `1`, and
    a `1` on the superdiagonal only ever joins two equal diagonal entries
    (so the matrix is a block-diagonal sum of Jordan blocks). -/
def IsJordanForm {m : Nat} (J : Matrix (Fin m) (Fin m) Rat) : Prop :=
  (∀ i j : Fin m, j.val < i.val → J i j = 0) ∧
  (∀ i j : Fin m, i.val + 1 < j.val → J i j = 0) ∧
  (∀ i : Fin m, (h : i.val + 1 < m) →
    let j : Fin m := ⟨i.val + 1, h⟩
    J i j = 0 ∨ J i j = 1) ∧
  (∀ i : Fin m, (h : i.val + 1 < m) →
    let j : Fin m := ⟨i.val + 1, h⟩
    J i j = 1 → J i i = J j j)

/-- **Jordan normal form correctness.**

    On success, `A` is similar to the returned Jordan matrix `J`: there exists
    an invertible matrix `P` with `P · J = A · P`. -/
theorem jordan_form_correct
    (matrix : Slice Rat) (n : Std.Usize) (result : Jordan)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h_j : (alloc.vec.Vec.deref result.j).length = (n.val : Nat) * (n.val : Nat))
    (h : jordan_form matrix n = ok result) :
    result.success = true →
      ∃ P : Matrix (Fin (n.val : Nat)) (Fin (n.val : Nat)) Rat,
        P.det ≠ 0 ∧
        P * (toMatrix (alloc.vec.Vec.deref result.j) (n.val : Nat) (n.val : Nat) h_j) =
          (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len) * P := by
  sorry

/-- **The returned matrix is in Jordan normal form.** -/
theorem jordan_is_jordan_form
    (matrix : Slice Rat) (n : Std.Usize) (result : Jordan)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h_j : (alloc.vec.Vec.deref result.j).length = (n.val : Nat) * (n.val : Nat))
    (h : jordan_form matrix n = ok result) :
    result.success = true →
      IsJordanForm (toMatrix (alloc.vec.Vec.deref result.j) (n.val : Nat) (n.val : Nat) h_j) := by
  sorry

/-- **Success criterion:** `jordan_form` succeeds exactly when the
    characteristic polynomial factors into linear factors with integer roots
    (equivalently, when every eigenvalue is an integer). -/
theorem jordan_form_success_iff
    (matrix : Slice Rat) (n : Std.Usize) (result : Jordan)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h_j : (alloc.vec.Vec.deref result.j).length = (n.val : Nat) * (n.val : Nat))
    (h : jordan_form matrix n = ok result) :
    result.success = true ↔
      ∃ roots : Fin (n.val : Nat) → Int,
        Matrix.charpoly (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len) =
          ∏ i : Fin (n.val : Nat), (Polynomial.X - Polynomial.C ((roots i : Rat))) := by
  sorry

/-- **Failure branch:** when the characteristic polynomial does not split into
    linear factors over the integers, `jordan_form` returns `success = false`
    and the all-zero n×n matrix. -/
theorem jordan_form_failure_zero
    (matrix : Slice Rat) (n : Std.Usize) (result : Jordan)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h_j : (alloc.vec.Vec.deref result.j).length = (n.val : Nat) * (n.val : Nat))
    (h : jordan_form matrix n = ok result) :
    result.success = false →
      toMatrix (alloc.vec.Vec.deref result.j) (n.val : Nat) (n.val : Nat) h_j = 0 := by
  sorry
