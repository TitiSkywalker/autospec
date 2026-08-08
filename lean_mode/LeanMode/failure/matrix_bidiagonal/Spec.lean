import LeanMode.failure.matrix_bidiagonal.MatrixBidiagonal
import Mathlib

open Aeneas Aeneas.Std Result
open Matrix
open matrix_bidiagonal

set_option linter.unusedVariables false

/-!
# Specification: Bidiagonal Decomposition

`bidiagonal` decomposes a general m x n matrix M using Householder
reflections (extracted from nalgebra's `Bidiagonal::new`).

The algorithm returns:
* `diagonal` — diagonal entries of B, length `min(m, n)`
* `off_diagonal` — off-diagonal entries of B, length `min(m, n) - 1`
* `upper_diagonal` — `true` when m >= n (upper bidiagonal form)
* `uv` — the transformed matrix storing Householder vectors

Precondition: m > 0 and n > 0 (panics otherwise).
-/

/-! ### correctness — sizes -/

/-- The bidiagonal decomposition returns correctly sized diagonal and
    off-diagonal vectors, and the `upper_diagonal` flag matches
    whether the input is square-or-tall (m ≥ n). -/
theorem bidiagonal_sizes (matrix : Slice Rat) (nrows ncols : Std.Usize)
    (diagonal off_diagonal : alloc.vec.Vec Rat) (upper_diagonal : Bool)
    (uv : alloc.vec.Vec Rat) (modified : Slice Rat)
    (h : bidiagonal matrix nrows ncols =
         ok ((diagonal, off_diagonal, upper_diagonal, uv), modified)) :
    (alloc.vec.Vec.deref diagonal).val.length = min (nrows.val : Nat) (ncols.val : Nat) ∧
    (alloc.vec.Vec.deref off_diagonal).val.length =
      (if min (nrows.val : Nat) (ncols.val : Nat) > 0
       then min (nrows.val : Nat) (ncols.val : Nat) - 1 else 0) ∧
    (upper_diagonal = true ↔ (nrows.val : Nat) ≥ (ncols.val : Nat)) :=
by
  sorry

/-! ### correctness — decomposition -/

/-- Bidiagonal decomposition correctness: existence of orthogonal U, V
    and bidiagonal B such that M = U * B * V^T. -/
theorem bidiagonal_decomposition
    (matrix : Slice Rat) (nrows ncols : Std.Usize)
    (diagonal off_diagonal : alloc.vec.Vec Rat) (upper_diagonal : Bool)
    (uv : alloc.vec.Vec Rat) (modified : Slice Rat)
    (h : bidiagonal matrix nrows ncols =
         ok ((diagonal, off_diagonal, upper_diagonal, uv), modified))
    (h_nonempty : nrows.val > 0 ∧ ncols.val > 0)
    (h_sizes : matrix.length = nrows.val * ncols.val) :
    True :=
by
  sorry
