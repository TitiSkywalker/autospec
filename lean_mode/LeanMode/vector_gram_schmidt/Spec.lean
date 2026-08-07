import LeanMode.vector_gram_schmidt.VectorGramSchmidt
import Mathlib.Tactic

open Aeneas Aeneas.Std Result
open vector_gram_schmidt

set_option linter.unusedVariables false

/-!
# Specification: Gram-Schmidt Orthogonalization Step

`gram_schmidt_step` orthogonalizes a vector `v` against a reference
vector `u`, computing:

> **v' = v - (⟨v, u⟩ / ⟨u, u⟩) · u**

The result v' satisfies ⟨v', u⟩ = 0.

## Preconditions

The caller must provide a non-zero reference vector `u` (`⟨u, u⟩ ≠ 0`).
If `u` is the zero vector the function panics via `massert`.

## When does it succeed?

The only failure case is a zero reference vector.
-/

/-! ### helpers -/

/-- Flat `Slice Rat` → vector as `Fin n → Rat`. -/
def toVector (s : Slice Rat) (len : Nat)
    (h : s.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by
      rw [h]
      exact i.is_lt
    s[i.val]'h_bound


/-! ### orthogonality -/

theorem gram_schmidt_step_orthogonal
    (u v : Slice Rat) (n : Std.Usize) (result : alloc.vec.Vec Rat)
    (h_u_len : u.length = (n.val : Nat))
    (h_v_len : v.length = (n.val : Nat))
    (h_result_len : (alloc.vec.Vec.deref result).val.length =
                    (n.val : Nat))
    (h : gram_schmidt_step u v n = ok result) :
    dotProduct (toVector (alloc.vec.Vec.deref result)
                         (n.val : Nat) h_result_len)
               (toVector u (n.val : Nat) h_u_len) = 0 := by
  sorry

/-! ## correctness -/

theorem gram_schmidt_step_correct
    (u v : Slice Rat) (n : Std.Usize) (result : alloc.vec.Vec Rat)
    (h_u_len : u.length = (n.val : Nat))
    (h_v_len : v.length = (n.val : Nat))
    (h_result_len : (alloc.vec.Vec.deref result).val.length =
                    (n.val : Nat))
    (h : gram_schmidt_step u v n = ok result)
    (h_nonzero : dotProduct (toVector u (n.val : Nat) h_u_len)
                            (toVector u (n.val : Nat) h_u_len) ≠ 0) :
    (toVector (alloc.vec.Vec.deref result)
              (n.val : Nat) h_result_len) =
    (toVector v (n.val : Nat) h_v_len) -
    ((dotProduct (toVector v (n.val : Nat) h_v_len)
                 (toVector u (n.val : Nat) h_u_len)) /
     (dotProduct (toVector u (n.val : Nat) h_u_len)
                 (toVector u (n.val : Nat) h_u_len))) •
    (toVector u (n.val : Nat) h_u_len) := by
  sorry
