import LeanMode.matrix_permanent.MatrixPermanent
import Mathlib

open Aeneas Aeneas.Std Result
open Matrix
open matrix_permanent

set_option linter.unusedVariables false

/-!
# Specification: Matrix Permanent (Ryser's formula)

`permanent a n` computes the permanent of the `n × n` integer matrix `a`
(row-major, flat) with Ryser's inclusion–exclusion formula

```text
  per(A) = (-1)ⁿ · Σ_{S ⊆ {0,…,n-1}} (-1)^{|S|} · Π_i (Σ_{j ∈ S} A[i][j])
```

The specification below is the *definitional* one: the permanent is the sum,
over all permutations `σ` of `{0,…,n-1}`, of the products `Π_i A[i][σ(i)]`.
The theorem says the two agree — i.e. Ryser's formula is correct — for every
integer matrix, including the empty one (`n = 0`, where both sides are `1`).
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

/-- The **permanent** of a square integer matrix:

    `per(A) = Σ_{σ ∈ S_n} Π_i A[i][σ(i)]`,

the determinant with the sign of each permutation dropped. -/
def Permanent {n : Nat} (A : Matrix (Fin n) (Fin n) Int) : Int :=
  ∑ σ : Equiv.Perm (Fin n), ∏ i : Fin n, A i (σ i)

/-! ### correctness -/

/-- **Permanent correctness.**

    If `permanent a n` returns `ok p` for an `n × n` integer matrix `a` given in
    row-major order, then `p` is the permanent of `a` — the sum over all `n!`
    permutations of the diagonal products. -/
theorem permanent_correct (matrix : Slice Int) (n : Std.Usize) (p : Int)
    (h_len : matrix.length = n.val * n.val)
    (h : permanent matrix n = ok p) :
    p = Permanent (toMatrix matrix n.val n.val h_len) := by
  sorry
