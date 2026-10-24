import LeanMode.matrix_charpoly.MatrixCharpoly
import Mathlib

open Aeneas Aeneas.Std Result
open Matrix
open matrix_charpoly

set_option linter.unusedVariables false

/-!
# Specification: n×n Matrix Characteristic Polynomial

`charpoly` computes the coefficients of the characteristic polynomial
`det(x·I − A)` via the Faddeev–LeVerrier algorithm, returning
`[c₀, c₁, …, cₙ]` (ascending degree, `cₙ = 1`) for a row-major n×n matrix.

The deep insight here is the **Cayley–Hamilton theorem**: the characteristic
polynomial annihilates the matrix (`p(A) = 0`).  We state it below, matching
our computed coefficients against Mathlib's `Matrix.charpoly`.
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

/-- **Characteristic polynomial correctness.**

    The returned coefficient `cs[k]` (ascending degree) equals the `k`-th
    coefficient of Mathlib's `Matrix.charpoly` of the input matrix. -/
theorem charpoly_correct
    (matrix : Slice Rat) (n : Std.Usize) (coeffs : alloc.vec.Vec Rat)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h_coeffs : (alloc.vec.Vec.deref coeffs).length = n.val + 1)
    (h : charpoly matrix n = ok coeffs) :
    ∀ k : Fin (n.val + 1),
      have hk : k.val < (alloc.vec.Vec.deref coeffs).length := by
        rw [h_coeffs]
        exact k.is_lt
      (alloc.vec.Vec.deref coeffs)[k.val]'hk =
        (Matrix.charpoly (toMatrix matrix (n.val : Nat) (n.val : Nat) h_len)).coeff k.val := by
  sorry

/-- **Cayley–Hamilton theorem** (deep insight).

    The characteristic polynomial computed by `charpoly` annihilates the
    matrix: `c₀·I + c₁·A + ⋯ + cₙ·Aⁿ = 0`. -/
theorem charpoly_cayley_hamilton
    (matrix : Slice Rat) (n : Std.Usize) (coeffs : alloc.vec.Vec Rat)
    (h_len : matrix.length = (n.val : Nat) * (n.val : Nat))
    (h_coeffs : (alloc.vec.Vec.deref coeffs).length = n.val + 1)
    (h : charpoly matrix n = ok coeffs) :
    let A := toMatrix matrix (n.val : Nat) (n.val : Nat) h_len
    (∑ k : Fin (n.val + 1),
      have hk : k.val < (alloc.vec.Vec.deref coeffs).length := by
        rw [h_coeffs]
        exact k.is_lt
      (alloc.vec.Vec.deref coeffs)[k.val]'hk • (A ^ k.val)) = 0 := by
  sorry
