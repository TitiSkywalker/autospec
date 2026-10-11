import LeanMode.string_kmp.StringKmp
import Mathlib

open Aeneas Aeneas.Std Result
open string_kmp

set_option linter.unusedVariables false

/-!
# Specification: Knuth–Morris–Pratt string search

Strings are byte sequences (`&[u8]`), which translate to `Slice Nat`.  A pattern
`pattern` *occurs* in `text` at index `i` when the suffix of `text` starting at
`i` begins with `pattern`.
-/

/-- `pattern` occurs in `text` at index `i`: `text[i..] = pattern ++ _`. -/
def occursAt (text pattern : List Nat) (i : Nat) : Prop :=
  ∃ t, text.drop i = pattern ++ t

/-! ## Specification theorems -/

/-- `kmp_search` returns the first occurrence of `pattern` in `text`, or `none`
    when there is no occurrence. -/
theorem kmp_search_correct (text pattern : Slice Nat) (result : Option Std.Usize)
    (h : kmp_search text pattern = ok result) :
    (∃ i, result = some i ∧ occursAt (text : List Nat) (pattern : List Nat) i.val ∧
        ∀ j, j < i.val → ¬ occursAt (text : List Nat) (pattern : List Nat) j) ∨
    (result = none ∧ ∀ i, ¬ occursAt (text : List Nat) (pattern : List Nat) i) := by
  sorry
