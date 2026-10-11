import LeanMode.string_rabin_karp.StringRabinKarp
import Mathlib

open Aeneas Aeneas.Std Result
open string_rabin_karp

set_option linter.unusedVariables false

/-!
# Specification: Rabin–Karp string search

Characters are integer-encoded as `u64` (→ `Nat`).  `pattern` *occurs* in `text`
at index `i` when the suffix of `text` starting at `i` begins with `pattern`.
The rolling hash is only a filter — a hash hit is re-checked byte-by-byte — so
the result is exact.
-/

/-- `pattern` occurs in `text` at index `i`: `text[i..] = pattern ++ _`. -/
def occursAt (text pattern : List Nat) (i : Nat) : Prop :=
  ∃ t, text.drop i = pattern ++ t

/-! ## Specification theorems -/

/-- `rabin_karp` returns the first occurrence of `pattern` in `text`, or `none` when there is no occurrence. -/
theorem rabin_karp_correct (text pattern : Slice Nat) (result : Option Std.Usize)
    (h : rabin_karp text pattern = ok result) :
    (∃ i, result = some i ∧ occursAt (text : List Nat) (pattern : List Nat) i.val ∧
        ∀ j, j < i.val → ¬ occursAt (text : List Nat) (pattern : List Nat) j) ∨
    (result = none ∧ ∀ i, ¬ occursAt (text : List Nat) (pattern : List Nat) i) := by
  sorry
