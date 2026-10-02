import LeanMode.array_max.VectorMax
import Mathlib

open Aeneas Aeneas.Std Result
open array_max

set_option linter.unusedVariables false

/-!
# Specification: Vector Max

`max input` returns the maximum element in the slice.
Returns `0` for an empty slice.
-/

/-- View a `Slice Rat` of known length as `Fin len → Rat`. -/
def toVec (s : Slice Rat) (len : Nat) (h : s.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound


/-- Correctness: if non-empty, `max input` is the maximum over `Fin n`.
    If empty, result is `0`. -/
theorem max_eq
    (input : Slice Rat) (result : Rat)
    (h_max : max input = ok result)
    (h_len : input.length = (Slice.len input).val) :
    let n := (Slice.len input).val
    if hn : n = 0 then
      result = 0
    else
      -- Fin n is nonempty because n ≠ 0
      have hpos : 0 < n := Nat.pos_of_ne_zero hn
      have h_ne : Finset.Nonempty (Finset.univ : Finset (Fin n)) := by
        refine ⟨⟨0, hpos⟩, Finset.mem_univ _⟩
      result = Finset.sup' (Finset.univ : Finset (Fin n)) h_ne (toVec input n h_len) := by
  sorry
