import LeanMode.integer_binomial.IntegerBinomial
import Mathlib

open Aeneas Aeneas.Std Result
open integer_binomial

set_option linter.unusedVariables false

/-!
# Specification: Binomial coefficient

`binomial n k` computes `C(n, k) = n! / (k! · (n-k)!)`.
-/

theorem binomial_correct (n k result : Nat)
    (h : binomial n k = ok result) :
    result = Nat.choose n k := by
  sorry
