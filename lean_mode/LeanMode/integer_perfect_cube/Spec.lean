import LeanMode.integer_perfect_cube.IntegerPerfectCube
import Mathlib

open Aeneas Aeneas.Std Result
open integer_perfect_cube

set_option linter.unusedVariables false

/-!
# Specification: Perfect cube test

`is_perfect_cube n` is `true` iff `n` is a perfect cube (∃k, k³ = n).
-/

theorem is_perfect_cube_correct (n : Nat) (result : Bool)
    (h : is_perfect_cube n = ok result) :
    result = true ↔ ∃ k : Nat, k ^ 3 = n := by
  sorry
