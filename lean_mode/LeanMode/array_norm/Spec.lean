import LeanMode.array_norm.ArrayNorm
import Mathlib

open Aeneas Aeneas.Std Result
open array_norm

set_option linter.unusedVariables false

/-!
# Specification: Vector Euclidean Norm

`vector_norm v n` computes the Euclidean (L²) norm of `v[0..n]`.
It is identified with mathlib's `‖·‖` on `Fin n → ℝ`.
-/

/-- View a `Slice ℝ` of known length as a vector `Fin n → ℝ`. -/
def toVec (s : Slice ℝ) (len : Nat) (h : s.length = len) : Fin len → ℝ :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound

/-- `vector_norm` equals mathlib's Euclidean norm `‖·‖`. -/
theorem vector_norm_eq_norm (v : Slice ℝ) (n : Std.Usize)
    (result : ℝ) (h : vector_norm v n = ok result)
    (h_len : v.length = (n.val : Nat)) :
    result = ‖toVec v (n.val : Nat) h_len‖ :=
by
  sorry

/-- Corollary: the norm is non-negative. -/
theorem vector_norm_nonneg (v : Slice ℝ) (n : Std.Usize)
    (result : ℝ) (h : vector_norm v n = ok result)
    (h_len : v.length = (n.val : Nat)) :
    result ≥ 0 :=
by
  rw [vector_norm_eq_norm v n result h h_len]
  exact norm_nonneg _

/-- Zero norm iff all components are zero. -/
theorem vector_norm_eq_zero_iff (v : Slice ℝ) (n : Std.Usize)
    (result : ℝ) (h : vector_norm v n = ok result)
    (h_len : v.length = (n.val : Nat)) :
    result = 0 ↔ ∀ i, toVec v (n.val : Nat) h_len i = 0 :=
by
  rw [vector_norm_eq_norm v n result h h_len]
  haveI : Fact (1 ≤ 2) := ⟨by norm_num⟩
  simp [norm_eq_zero, funext_iff]
