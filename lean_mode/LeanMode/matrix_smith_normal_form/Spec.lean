import LeanMode.matrix_smith_normal_form.MatrixSmithNormalForm
import Mathlib

open Aeneas Aeneas.Std Result
open Matrix
open matrix_smith_normal_form

set_option linter.unusedVariables false

/-!
# Specification: Smith Normal Form

`smith_normal_form a nrows ncols` returns the Smith normal form of the integer
matrix `a` together with the two unimodular transforms:

```text
  D = P · A · Q
```

where `A` is the input, `P` (`nrows × nrows`) and `Q` (`ncols × ncols`) have
determinant `±1`, and `D` (`nrows × ncols`) is diagonal with non-negative
entries satisfying the divisibility chain `d₀ | d₁ | …`.  The `dᵢ` are the
*invariant factors* of `A`.

The theorem below is the full statement of the normal-form theorem: the output
of the algorithm really is a Smith normal form of the input, reached by
unimodular row/column operations (which is what makes `dᵢ` a complete
isomorphism invariant of the cokernel `ℤ^ncols / A·ℤ^nrows`).
-/

/-! ### helpers -/

/-- Row-major flat slice → Mathlib `Matrix`:
    `(i, j) ↦ s[i * cols + j]`. -/
def toMatrix (s : Slice Int) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) Int :=
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

/-- The `k`-th entry of the main diagonal of an `nrows × ncols` matrix, for
    `k < min nrows ncols`. -/
def diagAt {nrows ncols : Nat} (D : Matrix (Fin nrows) (Fin ncols) Int)
    (k : Fin (min nrows ncols)) : Int :=
  D ⟨k.val, lt_of_lt_of_le k.is_lt (min_le_left nrows ncols)⟩
    ⟨k.val, lt_of_lt_of_le k.is_lt (min_le_right nrows ncols)⟩

/-- All entries off the main diagonal vanish. -/
def IsDiagonal {nrows ncols : Nat} (D : Matrix (Fin nrows) (Fin ncols) Int) : Prop :=
  ∀ (i : Fin nrows) (j : Fin ncols), i.val ≠ j.val → D i j = 0

/-- Unimodularity: an integer square matrix of determinant `±1`, i.e. an
    invertible matrix over `ℤ`. -/
def IsUnimodular {n : Nat} (M : Matrix (Fin n) (Fin n) Int) : Prop :=
  Matrix.det M = 1 ∨ Matrix.det M = -1

/-! ### correctness -/

/-- **Smith normal form correctness.**

    For an `nrows × ncols` integer matrix `a` (row-major, flat) the returned
    triple `(p, d, q)` satisfies `d = p · a · q` with `p`, `q` unimodular and
    `d` diagonal, non-negative on the diagonal and with the divisibility chain
    `dᵢ | dⱼ` for `i < j`. -/
theorem smith_normal_form_correct
    (matrix : Slice Int) (nrows ncols : Std.Usize) (result : SmithForm)
    (h_len : matrix.length = nrows.val * ncols.val)
    (h_p : (alloc.vec.Vec.deref result.p).length = nrows.val * nrows.val)
    (h_d : (alloc.vec.Vec.deref result.d).length = nrows.val * ncols.val)
    (h_q : (alloc.vec.Vec.deref result.q).length = ncols.val * ncols.val)
    (h : smith_normal_form matrix nrows ncols = ok result) :
    let P := toMatrix (alloc.vec.Vec.deref result.p) nrows.val nrows.val h_p
    let D := toMatrix (alloc.vec.Vec.deref result.d) nrows.val ncols.val h_d
    let Q := toMatrix (alloc.vec.Vec.deref result.q) ncols.val ncols.val h_q
    let A := toMatrix matrix nrows.val ncols.val h_len
    D = P * A * Q ∧
    IsDiagonal D ∧
    IsUnimodular P ∧
    IsUnimodular Q ∧
    (∀ k : Fin (min nrows.val ncols.val), 0 ≤ diagAt D k) ∧
    (∀ k l : Fin (min nrows.val ncols.val), k.val < l.val → diagAt D k ∣ diagAt D l) := by
  sorry
