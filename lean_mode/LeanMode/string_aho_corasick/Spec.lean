import LeanMode.string_aho_corasick.StringAhoCorasick
import Mathlib

open Aeneas Aeneas.Std Result
open string_aho_corasick

set_option linter.unusedVariables false

/-!
# Specification: Aho–Corasick multi-pattern matching

`aho_corasick text patterns offsets` returns a mask `matched` with
`matched[p] = true` iff pattern `p` (the slice `patterns[offsets[p]..offsets[p+1]]`)
occurs somewhere in `text`.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def uszList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- View a `Slice Bool` as a `List Bool`. -/
def maskList (s : Slice Bool) : List Bool := (s : List Bool)

/-- The `p`-th pattern, decoded from the flat `patterns`/`offsets` encoding. -/
def patternAt (patterns offsets : List Nat) (p : Nat) : List Nat :=
  (patterns.drop (offsets.getD p 0)).take
    (offsets.getD (p + 1) 0 - offsets.getD p 0)

/-- `pattern` occurs in `text` at index `i`: `text[i..] = pattern ++ _`. -/
def occursAt (text pattern : List Nat) (i : Nat) : Prop :=
  ∃ t, text.drop i = pattern ++ t

/-- `pattern` occurs somewhere in `text`. -/
def occurs (text pattern : List Nat) : Prop :=
  ∃ i, occursAt text pattern i

/-! ## Specification theorems -/

/-- `aho_corasick` marks pattern `p` as matched exactly when pattern `p`
    occurs in `text`. -/
theorem aho_corasick_correct (text patterns offsets : Slice Std.Usize)
    (result : alloc.vec.Vec Bool)
    (h : aho_corasick text patterns offsets = ok result) :
    (alloc.vec.Vec.deref result).length = (offsets : List Std.Usize).length - 1 ∧
    (∀ p, p < (offsets : List Std.Usize).length - 1 →
      (maskList (alloc.vec.Vec.deref result)).getD p false = true ↔
        occurs (uszList (text : List Std.Usize))
          (patternAt (uszList (patterns : List Std.Usize))
            (uszList (offsets : List Std.Usize)) p)) := by
  sorry
