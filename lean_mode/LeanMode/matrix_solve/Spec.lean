import LeanMode.matrix_solve.MatrixSolve
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_solve
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Triangular Linear-System Solvers

`solve_lower_triangular` / `solve_upper_triangular` solve the linear systems
`L · x = b` (L lower-triangular) and `U · x = b` (U upper-triangular) by forward
/ back substitution.

Both preconditions are carried as explicit hypotheses: the coefficient matrix
must be triangular (`h_lower` / `h_upper`) and its diagonal must be non-zero
(`h_diag`).  The Rust `assert!` additionally rejects a zero pivot at run time,
as a double-check.
-/

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

/-- Flat `Slice Rat` of length `n` → a vector `Fin n → Rat`. -/
def toVector (s : Slice Rat) (n : Nat) (h : s.length = n) : Fin n → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by
      rw [h]
      exact i.is_lt
    s[i.val]'h_bound

/-! ### correctness -/

/-- **Lower-triangular solve output length.**

    The returned solution vector has length n. -/
theorem solve_lower_triangular_length
    (matrix : Slice Rat) (b : Slice Rat) (n : Std.Usize) (x : alloc.vec.Vec Rat)
    (h : solve_lower_triangular matrix b n = ok x) :
    (alloc.vec.Vec.deref x).length = n.val :=
by
  sorry

/-- **Lower-triangular solve correctness.**

    If `matrix` is lower-triangular (`h_lower`) with a non-zero diagonal
    (`h_diag`) and `solve_lower_triangular matrix b n` returns `ok x`, then `x`
    satisfies `M · x = b` (where `M` is the matrix view of `matrix`). -/
theorem solve_lower_triangular_correct
    (matrix : Slice Rat) (b : Slice Rat) (n : Std.Usize) (x : alloc.vec.Vec Rat)
    (h : solve_lower_triangular matrix b n = ok x)
    (h_m : matrix.length = n.val * n.val)
    (h_lower : ∀ i j, i.val < j.val → (toMatrix matrix n.val n.val h_m) i j = 0)
    (h_diag : ∀ i, (toMatrix matrix n.val n.val h_m) i i ≠ 0)
    (h_b : b.length = n.val)
    (h_x : (alloc.vec.Vec.deref x).length = n.val) :
    (toMatrix matrix n.val n.val h_m) *ᵥ (toVector (alloc.vec.Vec.deref x) n.val h_x)
      = toVector b n.val h_b :=
by
  sorry

/-- **Upper-triangular solve output length.**

    The returned solution vector has length n. -/
theorem solve_upper_triangular_length
    (matrix : Slice Rat) (b : Slice Rat) (n : Std.Usize) (x : alloc.vec.Vec Rat)
    (h : solve_upper_triangular matrix b n = ok x) :
    (alloc.vec.Vec.deref x).length = n.val :=
by
  sorry

/-- **Upper-triangular solve correctness.**

    If `matrix` is upper-triangular (`h_upper`) with a non-zero diagonal
    (`h_diag`) and `solve_upper_triangular matrix b n` returns `ok x`, then `x`
    satisfies `M · x = b` (where `M` is the matrix view of `matrix`). -/
theorem solve_upper_triangular_correct
    (matrix : Slice Rat) (b : Slice Rat) (n : Std.Usize) (x : alloc.vec.Vec Rat)
    (h : solve_upper_triangular matrix b n = ok x)
    (h_m : matrix.length = n.val * n.val)
    (h_upper : ∀ i j, j.val < i.val → (toMatrix matrix n.val n.val h_m) i j = 0)
    (h_diag : ∀ i, (toMatrix matrix n.val n.val h_m) i i ≠ 0)
    (h_b : b.length = n.val)
    (h_x : (alloc.vec.Vec.deref x).length = n.val) :
    (toMatrix matrix n.val n.val h_m) *ᵥ (toVector (alloc.vec.Vec.deref x) n.val h_x)
      = toVector b n.val h_b :=
by
  sorry
