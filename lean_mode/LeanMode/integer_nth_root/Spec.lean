import LeanMode.integer_nth_root.IntegerNthRoot
import Mathlib

open Aeneas Aeneas.Std Result
open integer_nth_root

set_option linter.unusedVariables false

/-!
# Specification: Integer n-th root

`nth_root x n` computes `⌊ⁿ√x⌋`, the largest `r` with `rⁿ ≤ x < (r+1)ⁿ`
(requires `n ≥ 1`, enforced by `massert`).
-/

theorem nth_root_correct (x n result : Nat)
    (h : nth_root x n = ok result) :
    result ^ n ≤ x ∧ x < (result + 1) ^ n := by
  sorry
